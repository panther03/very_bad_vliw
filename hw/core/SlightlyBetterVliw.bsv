`define KONATA_ENABLE
`define DEBUG_ENABLE
`include "Logging.bsv"

import FIFO::*;
import FIFOF::*;
import SpecialFIFOs::*;
import RegFile::*;
import RVUtil::*;
import Vector::*;
`ifdef KONATA_ENABLE
import KonataHelper::*;
`endif
import Printf::*;
import Ehr::*;
import BranchUnit::*;
import Alu::*;
import MemUnit::*;
import MemTypes::*;
import MultiportRam::*;



interface RVIfc;
    method ActionValue#(IMem) getIReq();
    method Action getIResp(IMem a);
    method ActionValue#(Mem) getDReq();
    method Action getDResp(Mem a);
    method ActionValue#(Mem) getMMIOReq();
    method Action getMMIOResp(Mem a);
endinterface

typedef struct {
    // Current PC
    Bit#(32) pc;
    // Previous PC
    Bit#(32) ppc;
    // Epoch (for squashing branches)
    Bit#(1) epoch;
} FetchInfo deriving (Eq, FShow, Bits);

typedef struct {
    Bit#(5) rs1;
    Bit#(5) rs2;
    Bit#(5) rd;
} RegisterUsage deriving (Eq, FShow, Bits);

typedef struct {
    Bit#(32) rv1;
    Bit#(32) rv2;
} Operands deriving (Eq, FShow, Bits);

/////////////////////////
// Pipeline Registers //
///////////////////////

typedef struct {
    FetchInfo fi;
`ifdef KONATA_ENABLE
    KonataId k_id;
`endif
} F2D deriving (Eq, FShow, Bits);

typedef struct {
    FetchInfo fi;
    Vector#(4, DecodedInst) dis;
    Vector#(4, RegisterUsage) rus;
`ifdef KONATA_ENABLE
    KonataId k_id;
`endif
} D2I deriving (Eq, FShow, Bits);

typedef struct {
    FetchInfo fi;
    Vector#(4, DecodedInst) dis;
    Vector#(4, Operands) ops;
    Vector#(4, Bit#(5)) dests;
`ifdef KONATA_ENABLE
    KonataId k_id;
`endif
} I2E deriving (Eq, FShow, Bits);

typedef struct {
    Vector#(4, DecodedInst) dis;
    Vector#(4, Bit#(5)) dests;
    Bool poisoned;
`ifdef KONATA_ENABLE
    KonataId k_id;
`endif
} E2W deriving (Eq, FShow, Bits);

/////////////////////
// Implementation //
///////////////////

module mkVLIW(RVIfc);
    ////////////////////////////////////////
    // Interface with memory and devices //
    //////////////////////////////////////
    FIFO#(IMem) toImem <- mkBypassFIFO;
    FIFO#(IMem) fromImem <- mkBypassFIFO;
    FIFO#(Mem) toDmem <- mkFIFO;
    FIFO#(Mem) fromDmem <- mkBypassFIFO;
    FIFO#(Mem) toMMIO <- mkFIFO;
    FIFO#(Mem) fromMMIO <- mkBypassFIFO;

    ////////////////
    // CPU state //
    //////////////
    Ehr#(2, Bit#(32)) pc <- mkEhr(32'h0);
    Ehr#(2, Bit#(1)) epoch <- mkEhr(1'h0);

    MultiportRam#(4, Bit#(32)) rf <- mkMultiportRam(32'h0);
    MultiportRam#(8, Bool) sc <- mkMultiportRam(True);

    Reg#(Bool) starting <- mkReg(True);

    RWire#(Vector#(4, Bit#(5))) sc_insert_dsts <- mkRWire;
    RWire#(Vector#(4, Bit#(5))) sc_remove_dsts <- mkRWire;

    ///////////////////////
    // Functional Units //
    /////////////////////
    let memUnitInput = (interface MemUnitInput;
        interface toDmem = toDmem;
        interface fromDmem = fromDmem;
        interface toMMIO = toMMIO;
        interface fromMMIO = fromMMIO;
    endinterface);
    MemUnit mu <- mkMemUnit(memUnitInput, True);
    let branchUnitInput = (interface BranchUnitInput;
        interface extPC = pc; interface extEpoch = epoch; 
    endinterface);
    BranchUnit bu <- mkBranchUnit(branchUnitInput);
    Alu alu1 <- mkAlu();
    Alu alu2 <- mkAlu();

    //////////////////////
    // Pipeline stages //
    ////////////////////
    FIFO#(F2D) f2d <- mkFIFO;
    FIFO#(D2I) d2i <- mkFIFO;
    FIFO#(I2E) i2e <- mkFIFO;
    FIFO#(E2W) e2w <- mkFIFO;


    ////////////
    // Rules //
    //////////
    rule init if (starting);
        starting <= False;
    endrule

`ifdef KONATA_ENABLE
	// Code to support Konata visualization
    String dumpFile = "output.log";
    let lfh <- mkReg(InvalidFile);
	Reg#(KonataId) fresh_id <- mkReg(0);
	Reg#(KonataId) commit_id <- mkReg(0);

	FIFO#(KonataId) retired <- mkFIFO;
	FIFO#(KonataId) squashed <- mkFIFO;

    rule do_tic_logging;
        if (starting) begin
            let f <- $fopen(dumpFile, "w") ;
            lfh <= f;
            $fwrite(f, "Kanata\t0004\nC=\t1\n");
        end
		konataTic(lfh);
	endrule
`endif

`ifdef DEBUG_ENABLE
    Reg#(Bit#(32)) cyc <- mkReg(0);
    rule cyc_count_debug;
        cyc <= cyc + 1;
    endrule
`endif

    rule fetch if (!starting);
        Bit#(32) pc_next = pc[0] + 16;

        pc[0] <= pc_next;

        let req = IMem {byte_en: 0, addr: pc[0], data: ?};
        toImem.enq(req);

`ifdef KONATA_ENABLE
        // Trigger konata
        let iid <- fetch1Konata(lfh, fresh_id, 0);
        labelKonataLeft(lfh, iid, $format("0x%x: ", pc[0]));
`endif 

        `DEBUG_PRINT(("[cyc=%d] Fetch @ %x", cyc, pc[0]))

        f2d.enq(F2D {
`ifdef KONATA_ENABLE
            k_id: iid,
`endif
            fi: FetchInfo {
                ppc: pc[0],
                pc: pc_next,
                epoch: epoch[0]
            }
        });
    endrule

    rule decode if (!starting);
        let f2d_result = f2d.first();
        let imem_resp = fromImem.first();

        Vector#(4, Word) bundle = unpack(imem_resp.data);
        
        Vector#(4, DecodedInst) dis;
        Vector#(4, RegisterUsage) rus;    
        Bool all_rdy = True;

        for (Integer i = 0; i < 4; i = i + 1) begin
            let inst = bundle[i];
            let fields = getInstFields(inst);
            dis[i] = decodeInst(inst);
            rus[i] = RegisterUsage {
                rs1: fields.rs1,
                rs2: fields.rs2,
                rd: fields.rd
            };
            let rs1_rdy = (!dis[i].valid_rs1 || fields.rs1 == 0) || sc.search(fields.rs1);
            let rs2_rdy = (!dis[i].valid_rs2 || fields.rs2 == 0) || sc.search(fields.rs2);
            all_rdy = all_rdy && rs1_rdy && rs2_rdy;
        end
        
        if (all_rdy) begin
            fromImem.deq();
            f2d.deq();

            Vector#(4, Bit#(5)) dests;    
            for (Integer i = 0; i < 4; i = i + 1) begin
                if (rus[i].rd != 0 && dis[i].valid_rd) begin
                    dests[i] = rus[i].rd; 
                end else begin
                    dests[i] = 0;
                end
            end
            sc_insert_dsts.wset(dests);


`ifdef KONATA_ENABLE
            decodeKonata(lfh, f2d_result.k_id);
//            labelKonataLeft(lfh,f2d_result.k_id, $format("RD=%d | rf[RS1=%d]=%x | rf[RS2=%d]=%d", rd_idx, rs1_idx, rs1, rs2_idx, rs2));
`endif

            `DEBUG_PRINT(("[cyc=%d] Decode @ %x", cyc, f2d_result.fi.ppc))

            d2i.enq(D2I {
`ifdef KONATA_ENABLE
                k_id: f2d_result.k_id,
`endif
                fi: f2d_result.fi,
                dis: dis,
                rus: rus
            });
        end
    endrule

    rule issue if (!starting);
        let d2i_result = d2i.first();
        let rus = d2i_result.rus;
        d2i.deq();
        
        Vector#(4, Operands) ops;
        Vector#(4, Bit#(5)) dests;
        Vector#(4, Bool) sc_vals = replicate(False);
        for (Integer i = 0; i < 4; i = i + 1) begin
            // 0 is hard-wired to 0 val in RISC-V
            let ru = rus[i];
            let rv1 = (ru.rs1 == 0 ? 0 : rf.search(ru.rs1));
            let rv2 = (ru.rs2 == 0 ? 0 : rf.search(ru.rs2));
            ops[i] = Operands {
                rv1: rv1,
                rv2: rv2
            };
            dests[i] = ru.rd;
        end

        `DEBUG_PRINT(("[cyc=%d] Issue @ %x", cyc, d2i_result.fi.ppc))

`ifdef KONATA_ENABLE
        issueKonata(lfh, d2i_result.k_id);
        //labelKonataLeft(lfh,fetchedInstr.k_id, $format("RD=%d | rf[RS1=%d]=%x | rf[RS2=%d]=%d", rd_idx, rs1_idx, rs1, rs2_idx, rs2));
`endif

        i2e.enq(I2E {
`ifdef KONATA_ENABLE
            k_id: d2i_result.k_id,
`endif
            fi: d2i_result.fi,
            dis: d2i_result.dis,
            ops: ops,
            dests: dests
        });
    endrule

    rule execute if (!starting);
        let i2e_result = i2e.first();
        let dis = i2e_result.dis;
        let ops = i2e_result.ops;
        i2e.deq();

        `DEBUG_PRINT(("[cyc=%d] Execute @ %x", cyc, i2e_result.fi.ppc))

`ifdef KONATA_ENABLE
        // Mark instruction in konata
        let current_id = i2e_result.k_id;
    	executeKonata(lfh, current_id);
`endif

        let poisoned = False;
        // Detect squashed instructions. We poison them so we can 
        // simply drop the instructions in writeback, freeing the 
        // scoreboard entry as we would normally.
        if (epoch[0] != i2e_result.fi.epoch) begin
`ifdef KONATA_ENABLE
            squashed.enq(current_id);
`endif
            `DEBUG_PRINT(("Squashed %x", i2e_result.fi.ppc))
            poisoned = True;
        end else begin
            // Memory Unit
            if (dis[0].inst != 0) begin
                let imm = getImmediate(dis[0]);
                let fields = getInstFields(dis[0].inst);
                let funct3 = fields.funct3;
                mu.enq(MemRequest {
                    rv1: ops[0].rv1,
                    rv2: ops[0].rv2,
                    imm: imm,
                    inst: dis[0].inst,
                    funct3: funct3,
                    pc: i2e_result.fi.ppc
                });
            end

            // Branch Unit
            if (dis[1].inst != 0) begin
                let imm = getImmediate(dis[1]);
                bu.enq(BranchRequest {
                    rv1: ops[1].rv1,
                    rv2: ops[1].rv2,
                    imm: imm,
                    inst: dis[1].inst,
                    pc: i2e_result.fi.ppc
                });
            end

            // Arithmetic Unit #1
            if (dis[2].inst != 0) begin
                let imm = getImmediate(dis[2]);
                alu1.enq(AluRequest {
                    rv1: ops[2].rv1,
                    rv2: ops[2].rv2,
                    imm: imm,
                    inst: dis[2].inst,
                    pc: i2e_result.fi.ppc
                });
            end

            // Arithmetic Unit #2
            if (dis[3].inst != 0) begin
                let imm = getImmediate(dis[3]);
                alu2.enq(AluRequest {
                    rv1: ops[3].rv1,
                    rv2: ops[3].rv2,
                    imm: imm,
                    inst: dis[3].inst,
                    pc: i2e_result.fi.ppc
                });
            end
        end

        e2w.enq(E2W{ 
`ifdef KONATA_ENABLE
            k_id: current_id,
`endif
            dis: dis,
            dests: i2e_result.dests,
            poisoned: poisoned
        });
    endrule

    rule writeback if (!starting);
        let e2w_result = e2w.first();
        let dis = e2w_result.dis;
        let dests = e2w_result.dests;
        let poisoned = e2w_result.poisoned;
        e2w.deq();

        `DEBUG_PRINT(("[cyc=%d] Writeback", cyc))
`ifdef KONATA_ENABLE
        let current_id = e2w_result.k_id;
        writebackKonata(lfh,current_id);
        retired.enq(current_id);
`endif
        
        Vector#(4,Bit#(32)) rf_vals;
        Vector#(4,Bit#(5)) rf_dsts;
        Vector#(4,Bit#(5)) sc_dsts;
        if (dis[0].inst != 0) begin
            if (!poisoned) begin
                rf_vals[0] <- mu.deq();
                rf_dsts[0] = dests[0];
            end else begin
                rf_vals[0] = 0;
                rf_dsts[0] = 0;
            end
            sc_dsts[0] = dests[0];
        end else begin
            rf_vals[0] = 0;
            rf_dsts[0] = 0;
            sc_dsts[0] = 0;
        end

        if (dis[1].inst != 0) begin
            if (!poisoned) begin
                rf_vals[1] <- bu.deq();
                rf_dsts[1] = dests[1];
            end else begin
                rf_vals[1] = 0;
                rf_dsts[1] = 0;
            end
            sc_dsts[1] = dests[1];
        end else begin
            rf_vals[1] = 0;
            rf_dsts[1] = 0;
            sc_dsts[1] = 0;
        end

        if (dis[2].inst != 0) begin
            if (!poisoned) begin
                rf_vals[2] <- alu1.deq();
                rf_dsts[2] = dests[2];
            end else begin
                rf_vals[2] = 0;
                rf_dsts[2] = 0;
            end
            sc_dsts[2] = dests[2];
        end else begin
            rf_vals[2] = 0;
            rf_dsts[2] = 0;
            sc_dsts[2] = 0;
        end

        if (dis[3].inst != 0) begin 
            if (!poisoned) begin
                rf_vals[3] <- alu2.deq();
                rf_dsts[3] = dests[3];
            end else begin
                rf_vals[3] = 0;
                rf_dsts[3] = 0;
            end
            sc_dsts[3] = dests[3];
        end else begin
            rf_vals[3] = 0;
            rf_dsts[3] = 0;
            sc_dsts[3] = 0;
        end

        $display("Write ", fshow(rf_dsts), fshow(rf_vals));
        rf.set(rf_dsts, rf_vals);
        sc_remove_dsts.wset(sc_dsts);
    endrule

    rule updateScoreboard if (!starting);
        Vector#(4, Bit#(5)) remove_dsts = fromMaybe(replicate(5'h0), sc_remove_dsts.wget());
        Vector#(4, Bit#(5)) insert_dsts = fromMaybe(replicate(5'h0), sc_insert_dsts.wget());
        Vector#(8, Bit#(5)) dsts = append(remove_dsts, insert_dsts);
        // Marking busy (i.e. setting to False in SC) has higher priority than
        // setting to free'd (True). So Falses are in LSBs.
        Vector#(8, Bool) vals = unpack(8'h0F);
        sc.set(dsts, vals);
    endrule

	// ADMINISTRATION:

    `ifdef KONATA_ENABLE
    rule administrative_konata_commit;
		    retired.deq();
		    let f = retired.first();
		    commitKonata(lfh, f, commit_id);
	endrule
		
	rule administrative_konata_flush;
		    squashed.deq();
		    let f = squashed.first();
		    squashKonata(lfh, f);
	endrule
`endif
		
    method ActionValue#(IMem) getIReq();
		toImem.deq();
		return toImem.first();
    endmethod
    method Action getIResp(IMem a);
    	fromImem.enq(a);
    endmethod
    method ActionValue#(Mem) getDReq();
		toDmem.deq();
		return toDmem.first();
    endmethod
    method Action getDResp(Mem a);
		fromDmem.enq(a);
    endmethod
    method ActionValue#(Mem) getMMIOReq();
		toMMIO.deq();
		return toMMIO.first();
    endmethod
    method Action getMMIOResp(Mem a);
		fromMMIO.enq(a);
    endmethod
endmodule
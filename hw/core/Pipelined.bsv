//`define KONATA_ENABLE
//`define DEBUG_ENABLE
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


interface RVIfc#(numeric type n);
    method ActionValue#(Mem) getIReq();
    method Action getIResp(Mem a);
    method ActionValue#(Mem) getDReq();
    method Action getDResp(Mem a);
    method ActionValue#(Mem) getMMIOReq();
    method Action getMMIOResp(Mem a);
endinterface

interface Scoreboard;
    method Action insert(Bit#(5) dst);
    method Action remove(Bit#(5) dst);
    method ActionValue#(Bool) search(Bit#(5) src);
endinterface

module mkScoreboardBoolFlags(Scoreboard); 
    Vector#(32,Reg#(Bool)) ready <- replicateM(mkReg(True));

    method Action insert(Bit#(5) dst);
        ready[dst] <= False;
    endmethod

    method Action remove(Bit#(5) dst);
        ready[dst] <= True;
    endmethod

    method ActionValue#(Bool) search(Bit#(5) src);
        return ready[src];
    endmethod
endmodule

typedef struct { Bit#(32) pc;
                 Bit#(32) ppc;
                 Bit#(1) epoch; 
                 Bit#(1) thread_id;
`ifdef KONATA_ENABLE
                 KonataId k_id; // <- This is a unique identifier per instructions, for logging purposes
`endif
             } F2D deriving (Eq, FShow, Bits);

typedef struct { 
    DecodedInst dinst;
    Bit#(32) pc;
    Bit#(32) ppc;
    Bit#(1) epoch;
    Bit#(32) rv1; 
    Bit#(32) rv2; 
    Bit#(1) thread_id;
`ifdef KONATA_ENABLE
    KonataId k_id; // <- This is a unique identifier per instructions, for logging purposes
`endif
    } D2E deriving (Eq, FShow, Bits);

typedef struct { 
    DecodedInst dinst;
    Bool poisoned;
    Bit#(1) thread_id;
`ifdef KONATA_ENABLE
    KonataId k_id; // <- This is a unique identifier per instructions, for logging purposes
`endif
} E2W deriving (Eq, FShow, Bits);

//(* synthesize *)
module mkPipelined (RVIfc#(n));
    // Interface with memory and devices
    FIFO#(Mem) toImem <- mkBypassFIFO;
    FIFO#(Mem) fromImem <- mkBypassFIFO;
    FIFO#(Mem) toDmem <- mkFIFO;
    FIFO#(Mem) fromDmem <- mkBypassFIFO;
    FIFO#(Mem) toMMIO <- mkFIFO;
    FIFO#(Mem) fromMMIO <- mkBypassFIFO;

    Integer numThreads = valueOf(n);
    Vector#(n, Ehr#(2, Bit#(32))) pcs <- replicateM(mkEhr(32'h0));
    Vector#(n, Ehr#(2, Bit#(1))) epochs <- replicateM(mkEhr(1'h0));
    Vector#(n, Vector#(32, Reg#(Bit#(32)))) rfs <- replicateM(replicateM(mkReg(32'h0)));
    Vector#(n, Scoreboard) scs <- replicateM(mkScoreboardBoolFlags);

    RWire#(Bit#(6)) decodeSCPort <- mkRWire;

    Reg#(Bit#(1)) lastFetchedThread <- mkReg(fromInteger(numThreads - 1));

    let memUnitInput = (interface MemUnitInput;
        interface toDmem = toDmem;
        interface fromDmem = fromDmem;
        interface toMMIO = toMMIO;
        interface fromMMIO = fromMMIO;
    endinterface);
    MemUnit mu <- mkMemUnit(memUnitInput);
    let branchUnitInput = (interface BranchUnitInput;
        interface extPC = pcs[0]; interface extEpoch = epochs[0]; 
    endinterface);
    BranchUnit bu <- mkBranchUnit(branchUnitInput);
    Alu alu <- mkAlu();

    FIFO#(F2D) f2d <- mkFIFO;
    FIFO#(D2E) d2e <- mkFIFO;
    FIFOF#(E2W) e2w <- mkFIFOF;

    Reg#(Bool) starting <- mkReg(True);

    rule init if (starting);
        starting <= False;
        for (Integer i = 1; i < numThreads; i = i + 1) begin
            rfs[i][fromInteger(i)] <= pack(10);
        end
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

	for (Integer thread = 0; thread < numThreads; thread = thread + 1) begin
        Integer prevThread = (thread == 0) ? (numThreads-1) : (thread - 1);
        rule fetch if (!starting && (lastFetchedThread == fromInteger(prevThread)));
        // Fetch PC including bypassed result from execute
        Bit#(32) pc_next = pcs[thread][0] + 4;

        // Update PC register based on calculated next
        // This will include the bypassed jump target
        pcs[thread][0] <= pc_next; 

        // Mem should also initiate a request from the bypass
        let req = Mem {byte_en: 0, addr: pcs[thread][0], data: ?};
        toImem.enq(req);

`ifdef KONATA_ENABLE
        // Trigger konata
        let iid <- fetch1Konata(lfh, fresh_id, 0);
        labelKonataLeft(lfh, iid, $format("0x%x: ", pcs[thread][0]));
`endif

`ifdef DEBUG_ENABLE 
        $display("(cyc=%d) [Fetch] thread=0", cyc, fshow(pcs[thread][0]));
`endif

        // Enqueue to the next pipeline stage
        f2d.enq(F2D{
            pc: pc_next, // NEXT pc
            ppc: pcs[thread][0], // PREVIOUS pc
`ifdef KONATA_ENABLE
            k_id: iid,
`endif
            epoch: epochs[thread][0],
            thread_id: 0
        });
        lastFetchedThread <= fromInteger(thread);
        endrule
    end


    rule decode if (!starting);
        // Check for operands being ready without dequeueing.
        // RS1 and RS2 need to be ready for RAW hazards,
        // while we need to wait on RD for a WAR hazard,
        // which here is only applicable when two insructions write to the same
        // register back-to-back and then after the register is read.
        let resp = fromImem.first();
        let instr = resp.data;
        let decodedInst = decodeInst(resp.data);
        let fetchedInstr = f2d.first();
        let thread = fetchedInstr.thread_id;
        let rs1_idx = getInstFields(instr).rs1;
        let rs2_idx = getInstFields(instr).rs2;
        let rd_idx = getInstFields(instr).rd;
        let rs1_rdy <- scs[thread].search(rs1_idx);
        let rs2_rdy <- scs[thread].search(rs2_idx);
        let valid_rs1 = True;
        let valid_rs2 = True;
     
        if ((rs1_rdy || !valid_rs1) && (rs2_rdy || !valid_rs2)) begin
            // Dequeue IMEM result with pipeline register, keeping them in-sync
            fromImem.deq();
            f2d.deq();

`ifdef DEBUG_ENABLE
            $display("(cyc=%d) [Decode] ", cyc, fshow(thread));
`endif

            // 0 is hard-wired to 0 val in RISC-V
            let rs1 = (rs1_idx == 0 ? 0 : rfs[thread][rs1_idx]);
            let rs2 = (rs2_idx == 0 ? 0 : rfs[thread][rs2_idx]);

            // RD is now busy in the scoreboard
            if (rd_idx != 0 && decodedInst.valid_rd) begin
                decodeSCPort.wset({thread,rd_idx});
                // $display("(cyc=%d) inserting %d", cyc, rd_idx);
            end

`ifdef KONATA_ENABLE
                decodeKonata(lfh, fetchedInstr.k_id);
                labelKonataLeft(lfh,fetchedInstr.k_id, $format("RD=%d | rf[RS1=%d]=%x | rf[RS2=%d]=%d", rd_idx, rs1_idx, rs1, rs2_idx, rs2));
`endif

            // Ready to execute; enqueue to next pipeline stage
            d2e.enq(D2E{
                dinst: decodedInst,
                pc: fetchedInstr.pc,
                ppc: fetchedInstr.ppc,
`ifdef KONATA_ENABLE
                k_id: fetchedInstr.k_id,
`endif
                epoch: fetchedInstr.epoch,
                rv1: rs1,
                rv2: rs2,
                thread_id: thread
            });
        end
    endrule

    rule scoreboardInsert if (!starting);
        if (isValid(decodeSCPort.wget())) begin
            Bit#(6) portVal = fromMaybe(6'h0, decodeSCPort.wget());
            scs[portVal[5]].insert(portVal[4:0]);
        end
    endrule

    rule execute if (!starting);
        // Dequeue from previous pipeline stage
        let decodedInstr = d2e.first();
        d2e.deq();
        let dInst = decodedInstr.dinst;
        let thread = decodedInstr.thread_id;

`ifdef KONATA_ENABLE
        // Mark instruction in konata
        let current_id = decodedInstr.k_id;
    	executeKonata(lfh, current_id);
`endif
        `DEBUG_PRINT(("(cyc=%d) [Execute] ", cyc, fshow(thread)))

		let imm = getImmediate(dInst);
        let poisoned = False;
        let fields = getInstFields(dInst.inst);
        let funct3 = fields.funct3;
        let rv1 = decodedInstr.rv1;
        let rv2 = decodedInstr.rv2;
		
        let instr_pc = decodedInstr.ppc; // we reference from the CURRENT (i.e. previous) PC

        // Detect squashed instructions. We poison them so we can 
        // simply drop the instructions in writeback, freeing the 
        // scoreboard entry as we would normally.
        if (epochs[thread][0] != decodedInstr.epoch) begin
`ifdef KONATA_ENABLE
            squashed.enq(current_id);
`endif
            poisoned = True;
        end else if (isMemoryInst(dInst)) begin
            mu.enq(MemRequest {
                rv1: rv1,
                rv2: rv2,
                imm: imm,
                inst: dInst.inst,
                funct3: funct3,
                pc: instr_pc
            });
        end else if (isControlInst(dInst)) begin
`ifdef KONATA_ENABLE
            labelKonataLeft(lfh,current_id, $format(" (CTRL)"));
`endif
            bu.enq(BranchRequest {
                rv1: rv1,
                rv2: rv2,
                imm: imm,
                inst: dInst.inst,
                pc: instr_pc
            });
		end else begin 
`ifdef KONATA_ENABLE
            labelKonataLeft(lfh,current_id, $format(" (ALU)"));
`endif
            alu.enq(AluRequest {
                rv1: rv1,
                rv2: rv2,
                imm: imm,
                inst: dInst.inst,
                pc: instr_pc
            });
		end

        e2w.enq(E2W{ 
            dinst: dInst,
`ifdef KONATA_ENABLE
            k_id: current_id,
`endif
            thread_id: thread,
            poisoned: poisoned
        });
        
    endrule

    rule writeback if (!starting);
        // Dequeue from previous pipeline stage
        // $display("E2W: %d; memResp: %d", e2w.notEmpty(), fromDmem.notEmpty());
        let executedInstr = e2w.first();
        e2w.deq();
        let dInst = executedInstr.dinst;
        let thread = executedInstr.thread_id;
        let poisoned = executedInstr.poisoned;
        let fields = getInstFields(dInst.inst);

        Bit#(32) data = ?;
        if (!poisoned) begin
            if (isMemoryInst(dInst)) begin
                data <- mu.deq();
            end else if (isControlInst(dInst)) begin
                data <- bu.deq();
            end else begin
                data <- alu.deq();
            end

`ifdef KONATA_ENABLE
            let current_id = executedInstr.k_id;
            writebackKonata(lfh,current_id);
            retired.enq(current_id);

            if (dInst.valid_rd) begin
              labelKonataLeft(lfh,current_id, $format("RF [%d]=%d", fields.rd, data));
            end
`endif

`ifdef DEBUG_ENABLE
            $display("(cyc=%d) [Writeback] data=%d t=", cyc, data, fshow(thread));
`endif
        end

        if (dInst.valid_rd) begin
            let rd_idx = fields.rd;
            if (rd_idx != 0) begin 
                if (!poisoned) begin 
                    rfs[thread][rd_idx] <= data;
                end
                // $display("(cyc=%d) removing %d", cyc, rd_idx);
                scs[thread].remove(rd_idx);
            end
        end

        // TODO: fix this fault logic so bluespec doesnt complain
        //if (!dInst.legal) begin
		//	if (debug) $display("[Writeback] Illegal Inst, Drop and fault: ", fshow(dInst));
		//	pc[0] <= 0;	// Fault
	    //end

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
		
    method ActionValue#(Mem) getIReq();
		toImem.deq();
		return toImem.first();
    endmethod
    method Action getIResp(Mem a);
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
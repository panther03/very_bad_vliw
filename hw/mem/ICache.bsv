// SINGLE CORE ASSOIATED CACHE -- stores words

import BRAM::*;
import FIFO::*;
import FIFOF::*;
import SpecialFIFOs::*;
import Ehr::*;
import Vector::*;
import MemTypes::*;

Bool debug = False;

interface L1ICAU;
    method ActionValue#(HitMissType) req(ICacheReq c);
    method ActionValue#(L1TaggedLine) resp;
    method Action update(L1LineIndex index, LineData data, L1LineTag tag, Bool dirty);
endinterface

module mkL1ICAU(L1ICAU); 
    Vector#(TExp#(7), Reg#(L1LineTag)) tagStore <- replicateM(mkReg(0));
    Vector#(TExp#(7), Reg#(Bool)) validStore <- replicateM(mkReg(False));
    BRAM_Configure cfg = defaultValue();
    cfg.loadFormat = tagged Binary "zero.mem";
    BRAM1Port#(Bit#(7), LineData) dataStore <- mkBRAM1Server(cfg);
    BRAM1Port#(Bit#(7), Bool) dirtyStore <- mkBRAM1Server(cfg);
    FIFO#(L1LineTag) tagFifo <- mkFIFO;
    
    method ActionValue#(HitMissType) req(ICacheReq c);
        let pa = parseL1IAddress(c.addr);
        let tag = tagStore[pa.index];
        let valid = validStore[pa.index];
        if (debug) $display("ind: %d, valid: ", pa.index, fshow(valid));
        let hit = tag == pa.tag && valid;
        if (hit) begin
            tagFifo.enq(tag);
            dataStore.portA.request.put(BRAMRequest{
                write: False,
                responseOnWrite: False,
                address: pa.index,
                datain: ?
            });
            // we don't care if it's dirty or not, but we do this
            // just so we can dequeue from both in resp
            dirtyStore.portA.request.put(BRAMRequest{
                write: False,
                responseOnWrite: False,
                address: pa.index,
                datain: ?
            });
            return LdHit;
        end else begin
            tagFifo.enq(tag);
            dirtyStore.portA.request.put(BRAMRequest{
                write: False,
                responseOnWrite: False,
                address: pa.index,
                datain: ?
            });
            dataStore.portA.request.put(BRAMRequest{
                write: False,
                responseOnWrite: False,
                address: pa.index,
                datain: ?
            });
            return Miss;
        end
    endmethod

    method ActionValue#(L1TaggedLine) resp;
        let isDirty <- dirtyStore.portA.response.get();
        let data <- dataStore.portA.response.get();
        tagFifo.deq(); 
        let tag = tagFifo.first();
        return L1TaggedLine { data: data, isDirty: isDirty, tag: tag };
    endmethod

    // TODO: do update() and req() need to be called in the same cycle?
    // if so, how will we manage the read-after write behavior?
    // does a 1-port BRAM even have a separate port for reading and writing?
    method Action update(L1LineIndex index, LineData data, L1LineTag tag, Bool dirty);
        tagStore[index] <= tag;
        validStore[index] <= True;
        if (debug) $display("Updating line @ %d", index, fshow(data));
        dirtyStore.portA.request.put(BRAMRequest{
            write: True,
            responseOnWrite: False,
            address: index,
            datain: dirty
        });
        dataStore.portA.request.put(BRAMRequest{
            write: True,
            responseOnWrite: False,
            address: index,
            datain: data
        });
    endmethod

endmodule

typedef struct {
    ICacheReq req;
    Bool hit;
} HitMissCacheReq deriving (Eq, FShow, Bits, Bounded);

// Notice the asymmetry in this interface, as mentioned in lecture.
// The processor thinks in 32 bits, but the other side thinks in 512 bits.
interface ICache;
    method Action putFromProc(ICacheReq e);
    method ActionValue#(IWord) getToProc();
    method ActionValue#(MainMemReq) getToMem();
    method Action putFromMem(MainMemResp e);
endinterface

(* synthesize *)
module mkICache(ICache);

    L1ICAU cau <- mkL1ICAU();

    FIFO#(IWord) hitQ <- mkBypassFIFO;
    FIFO#(HitMissCacheReq) currReqQ <- mkPipelineFIFO;
    FIFO#(MainMemReq) lineReqQ <- mkFIFO;
    FIFO#(MainMemResp) lineRespQ <- mkFIFO;

    Reg#(CacheState) state <- mkReg(WaitCAUResp);
    Reg#(Bit#(32)) cyc <- mkReg(0);

    rule cyc_count_debug if (debug);
        cyc <= cyc + 1;
    endrule

    rule handleCAUResponse if (state == WaitCAUResp);
        let currReq = currReqQ.first();
        let pa = parseL1IAddress(currReq.req.addr);
        if (currReq.hit) begin 
            IWord word = unpack(0);
            let x <- cau.resp();
            Vector#(4, IWord) line = unpack(x.data);
            word = line[pa.offset];
            if (debug) begin 
                $display("(cyc=%d) [Load Hit 2] Tag=%d Index=%d Offset=%d Data=%d", cyc, pa.tag, pa.index, pa.offset, word);
            end
            // If it's not a load, word value is a dont care, we just do this so CPU gets result
            currReqQ.deq();
            hitQ.enq(word);
        end else begin
            let x <- cau.resp();
            if (x.isDirty) begin
                // dirty line, need to evict and write to LLC
                lineReqQ.enq(MainMemReq {
                    write: 1'b1,
                    addr: {x.tag, pa.index},
                    data: x.data
                });
                state <= SendReq;
                if (debug) begin 
                    $display("(cyc=%d) [Dirty Miss] Tag=%d Index=%d Offset=%d (Replace Tag)=%d", cyc, pa.tag, pa.index, pa.offset, x.tag);
                end
            end else begin
                lineReqQ.enq(MainMemReq {
                    write: 1'b0,
                    addr: currReq.req.addr[31:6],
                    data: ?
                });
                if (debug) begin 
                    $display("(cyc=%d) [Clean Miss] Tag=%d Index=%d Offset=%d", cyc, pa.tag, pa.index, pa.offset);
                end
                state <= WaitDramResp;
            end
        end
    endrule

    rule handleWriteback if (state == SendReq);
        let currReq = currReqQ.first();
        let pa = parseL1IAddress(currReq.req.addr);
        if (debug) begin
            $display("(cyc=%d) [WB->DRAM]", cyc);
        end
        if (currReq.hit) begin
            $display("Sanity check failed, handling writeback for a hit request?");
        end
        lineReqQ.enq(MainMemReq {
            write: 1'b0,
            addr: currReq.req.addr[31:6],
            data: ?
        });
        state <= WaitDramResp;
    endrule

    rule handleDramResponse if (state == WaitDramResp);
        // Grab response from memory and the request we have been handling.
        let line = lineRespQ.first(); lineRespQ.deq();
        let currReq = currReqQ.first(); currReqQ.deq();
        let pa = parseL1IAddress(currReq.req.addr);

        if (debug) begin
            $display("(cyc=%d) [DRAM->CACHE]", cyc);
        end

        Vector#(4, IWord) line_vec = unpack(line);
        let word = line_vec[pa.offset];
        let dirty = False;
        // Always enqueue the word. If it's a store, we don't
        // actually care about the result, just that we got one.
        hitQ.enq(word);
        // Update line in CAU
        cau.update(pa.index, pack(line_vec), pa.tag, dirty);
        state <= WaitCAUResp;
    endrule

    method Action putFromProc(ICacheReq e);
        let hitMissResult <- cau.req(e);
        let pa = parseL1IAddress(e.addr);
        case (hitMissResult)
            LdHit: begin
                if (debug) $display("(cyc=%d) [Load Hit  ] Tag=%d Index=%d Offset=%d", cyc, pa.tag, pa.index, pa.offset);
                currReqQ.enq(HitMissCacheReq{req: e, hit: True});
            end
            // StHit don't need to do anything
            StHit: begin
                if (debug) $display("(cyc=%d) [St Hit    ] Tag=%d Index=%d Offset=%d WB=%d Data=%d", cyc, pa.tag, pa.index, pa.offset, e.data);
                currReqQ.enq(HitMissCacheReq{req: e, hit: True});
            end
            Miss: begin
                if (debug) begin 
                    $display("(cyc=%d) [Load Miss ] Tag=%d Index=%d Offset=%d", cyc, pa.tag, pa.index, pa.offset);
                end
                currReqQ.enq(HitMissCacheReq{req: e, hit: False});
            end
        endcase
    endmethod
        
    method ActionValue#(IWord) getToProc();
        hitQ.deq(); return hitQ.first();
    endmethod
        
    method ActionValue#(MainMemReq) getToMem();
        lineReqQ.deq(); return lineReqQ.first();
    endmethod
        
    method Action putFromMem(MainMemResp e);
        lineRespQ.enq(e);
    endmethod
endmodule

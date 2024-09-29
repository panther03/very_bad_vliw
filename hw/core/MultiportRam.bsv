// system imports
import BRAM::*;
import FIFO::*;
import FIFOF::*;
import SpecialFIFOs::*;
import Ehr::*;
import Vector::*;

// assume only 5-bit address for now
interface MultiportRam#(numeric type numPorts, type a);

    //interface Vector#(n, ScoreboardPort) ports;
    method Action set(Vector#(numPorts, Bit#(5)) dsts, Vector#(numPorts, a) vals);
    method a search(Bit#(5) src);
endinterface

module mkMultiportRam #(a initVal) (MultiportRam#(numPorts, a)) provisos (Bits#(a,sa));
    Vector#(32,Reg#(a)) ram <- replicateM(mkReg(initVal));

    method Action set(Vector#(numPorts, Bit#(5)) dsts, Vector#(numPorts, a) vals);
        for (Integer i = 0; i < 32; i = i + 1) begin
            a iVal = ram[i];
            for (Integer p = 0; p < valueOf(numPorts); p = p + 1) begin  
                if (dsts[p] == fromInteger(i)) begin
                    iVal = vals[p];
                end
            end 
            ram[i] <= iVal;
        end
    endmethod

    method a search(Bit#(5) src);
        return ram[src];
    endmethod
endmodule
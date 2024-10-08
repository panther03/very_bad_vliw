import Core::*;
module mkSim(Empty);
    Core core <- mkCore(0, False);

    rule finishSim;
        if (core.getFinished()) $finish;
    endrule

    rule handleBusReq;
        let x <- core.getBusReq();

        if (x.write_enable) $fdisplay(stderr, "Bus Request: store %d = %d", x.addr, x.data);
        else begin
            core.putBusResp(0);
        end
    endrule

    
endmodule

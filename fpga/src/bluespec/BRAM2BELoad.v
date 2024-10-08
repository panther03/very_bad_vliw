`ifdef ALTERA_RESERVED_QIS
`ifdef BSV_ASSIGNMENT_DELAY
`else
 `define BSV_ASSIGNMENT_DELAY
`endif

// Dual-Ported BRAM (READ FIRST) with byte enables and ability to load from a file
module BRAM2BELoad(CLKA,
                   ENA,
                   WEA,
                   ADDRA,
                   DIA,
                   DOA,
                   CLKB,
                   ENB,
                   WEB,
                   ADDRB,
                   DIB,
                   DOB
                  );

   parameter                      FILENAME   = "";
   parameter                      PIPELINED  = 0;
   parameter                      ADDR_WIDTH = 1;
   parameter                      DATA_WIDTH = 1;
   parameter                      CHUNKSIZE  = 1;
   parameter                      WE_WIDTH   = 1;
   parameter                      MEMSIZE    = 1;
   parameter                      BINARY     = 0;

   input                          CLKA;
   input                          ENA;
   input [WE_WIDTH-1:0]           WEA;
   input [ADDR_WIDTH-1:0]         ADDRA;
   input [DATA_WIDTH-1:0]         DIA;
   output [DATA_WIDTH-1:0]        DOA;

   input                          CLKB;
   input                          ENB;
   input [WE_WIDTH-1:0]           WEB;
   input [ADDR_WIDTH-1:0]         ADDRB;
   input [DATA_WIDTH-1:0]         DIB;
   output [DATA_WIDTH-1:0]        DOB;


   wire                           RENA = ENA & !(|WEA);
   wire                           WENA = ENA & |WEA;
   wire                           RENB = ENB & !(|WEB);
   wire                           WENB = ENB & |WEB;
      
   altsyncram
     #(
       .width_a                            (DATA_WIDTH),
       .widthad_a                          (ADDR_WIDTH),
       .numwords_a                         (MEMSIZE),
       .outdata_reg_a                      ((PIPELINED) ? "CLOCK0" : "UNREGISTERED"),
       .address_aclr_a                     ("NONE"),
       .outdata_aclr_a                     ("NONE"),
       .indata_aclr_a                      ("NONE"),
       .wrcontrol_aclr_a                   ("NONE"),
       .byteena_aclr_a                     ("NONE"),
       .width_byteena_a                    (WE_WIDTH),

       .width_b                            (DATA_WIDTH),
       .widthad_b                          (ADDR_WIDTH),
       .numwords_b                         (MEMSIZE),
       .rdcontrol_reg_b                    ("CLOCK1"),//
       .address_reg_b                      ("CLOCK1"),//
       .outdata_reg_b                      ((PIPELINED) ? "CLOCK1" : "UNREGISTERED"),
       .outdata_aclr_b                     ("NONE"),//
       .rdcontrol_aclr_b                   ("NONE"),//
       .indata_reg_b                       ("CLOCK1"),//
       .wrcontrol_wraddress_reg_b          ("CLOCK1"),//
       .byteena_reg_b                      ("CLOCK1"),//
       .indata_aclr_b                      ("NONE"),//
       .wrcontrol_aclr_b                   ("NONE"),//
       .address_aclr_b                     ("NONE"),//
       .byteena_aclr_b                     ("NONE"),//
       .width_byteena_b                    (WE_WIDTH),

       .clock_enable_input_a               ("BYPASS"),
       .clock_enable_output_a              ("BYPASS"),
       .clock_enable_input_b               ("BYPASS"),
       .clock_enable_output_b              ("BYPASS"),

       .clock_enable_core_a                ("USE_INPUT_CLKEN"),//
       .clock_enable_core_b                ("USE_INPUT_CLKEN"),//
       .read_during_write_mode_port_a      ("DONT_CARE"),
       .read_during_write_mode_port_b      ("DONT_CARE"),

       .enable_ecc                         ("FALSE"),//
       .width_eccstatus                    (3),//
       .ecc_pipeline_stage_enabled         ("FALSE"),//

       .operation_mode                     ("BIDIR_DUAL_PORT"),
       .byte_size                          (CHUNKSIZE),//
       .read_during_write_mode_mixed_ports ("DONT_CARE"),//
       .ram_block_type                     ("AUTO"),//
       .init_file                          (FILENAME),//
       .init_file_layout                   ("PORT_A"),//
       .maximum_depth                      (MEMSIZE), // number of elements in memory
       .intended_device_family             ("Stratix"),//
       .lpm_hint                           ("ENABLE_RUNTIME_MOD=NO"),
       .lpm_type                           ("altsyncram"),//
       .implement_in_les                   ("OFF"), //
       .power_up_uninitialized             ("FALSE")
       )
   RAM
     (
      .wren_a                              (WENA),
      .rden_a                              (RENA),
      .data_a                              (DIA),
      .address_a                           (ADDRA),
      .clock0                              (CLKA),
      .clocken0                            (1'b1),
      .clocken1                            (1'b1),
      .aclr0                               (1'b0),
      .byteena_a                           (WEA),
      .addressstall_a                      (1'b0),
      .q_a                                 (DOA),

      .wren_b                              (WENB),
      .rden_b                              (RENB),
      .data_b                              (DIB),
      .address_b                           (ADDRB),
      .clock1                              (CLKB),
      .clocken2                            (1'b1),
      .clocken3                            (1'b1),
      .aclr1                               (1'b0),
      .byteena_b                           (WEB),
      .addressstall_b                      (1'b0),
      .q_b                                 (DOB),

      .eccstatus                           ()
      );

endmodule // BRAM2BELoad
`else

`ifdef BSV_ASSIGNMENT_DELAY
`else
 `define BSV_ASSIGNMENT_DELAY
`endif

// Dual-Ported BRAM (WRITE FIRST) with byte enables and ability to load from a file
module BRAM2BELoad(CLKA,
                   ENA,
                   WEA,
                   ADDRA,
                   DIA,
                   DOA,
                   CLKB,
                   ENB,
                   WEB,
                   ADDRB,
                   DIB,
                   DOB
                  );

   parameter                      FILENAME   = "";
   parameter                      PIPELINED  = 0;
   parameter                      ADDR_WIDTH = 1;
   parameter                      DATA_WIDTH = 1;
   parameter                      CHUNKSIZE  = 1;
   parameter                      WE_WIDTH   = 1;
   parameter                      MEMSIZE    = 1;
   parameter                      BINARY     = 0;

   input                          CLKA;
   input                          ENA;
   input [WE_WIDTH-1:0]           WEA;
   input [ADDR_WIDTH-1:0]         ADDRA;
   input [DATA_WIDTH-1:0]         DIA;
   output [DATA_WIDTH-1:0]        DOA;

   input                          CLKB;
   input                          ENB;
   input [WE_WIDTH-1:0]           WEB;
   input [ADDR_WIDTH-1:0]         ADDRB;
   input [DATA_WIDTH-1:0]         DIB;
   output [DATA_WIDTH-1:0]        DOB;

   reg [DATA_WIDTH-1:0]           RAM[0:MEMSIZE-1] /* synthesis syn_ramstyle="no_rw_check" */ ;
   reg [DATA_WIDTH-1:0]           DOA_R;
   reg [DATA_WIDTH-1:0]           DOA_R2;
   reg [DATA_WIDTH-1:0]           DOB_R;
   reg [DATA_WIDTH-1:0]           DOB_R2;

   // synopsys translate_off
   initial
   begin : init_block
`ifdef BSV_NO_INITIAL_BLOCKS
`else
      DOA_R  = { ((DATA_WIDTH+1)/2) { 2'b10 } };
      DOA_R2 = { ((DATA_WIDTH+1)/2) { 2'b10 } };
      DOB_R  = { ((DATA_WIDTH+1)/2) { 2'b10 } };
      DOB_R2 = { ((DATA_WIDTH+1)/2) { 2'b10 } };
`endif // !`ifdef BSV_NO_INITIAL_BLOCKS
   end
   // synopsys translate_on

   initial
   begin : init_rom_block
      if (BINARY)
        $readmemb(FILENAME, RAM, 0, MEMSIZE-1);
      else
        $readmemh(FILENAME, RAM, 0, MEMSIZE-1);
   end
   
   // PORT A
   generate
      genvar i;
      for(i = 0; i < WE_WIDTH; i = i + 1) begin: porta_we
         always @(posedge CLKA) begin
            if (ENA) begin
               if (WEA[i]) begin
                  RAM[ADDRA][((i+1)*CHUNKSIZE)-1 : i*CHUNKSIZE] <= `BSV_ASSIGNMENT_DELAY DIA[((i+1)*CHUNKSIZE)-1 : i*CHUNKSIZE];
                  DOA_R[((i+1)*CHUNKSIZE)-1 : i*CHUNKSIZE]      <= `BSV_ASSIGNMENT_DELAY DIA[((i+1)*CHUNKSIZE)-1 : i*CHUNKSIZE];
               end
               else begin
                  DOA_R[((i+1)*CHUNKSIZE)-1 : i*CHUNKSIZE]      <= `BSV_ASSIGNMENT_DELAY RAM[ADDRA][((i+1)*CHUNKSIZE)-1 : i*CHUNKSIZE];
               end
            end
         end
      end
   endgenerate

   // PORT B
   generate
      genvar k;
      for(k = 0; k < WE_WIDTH; k = k + 1) begin: portb_we
         always @(posedge CLKB) begin
            if (ENB) begin
               if (WEB[k]) begin
                  RAM[ADDRB][((k+1)*CHUNKSIZE)-1 : k*CHUNKSIZE] <= `BSV_ASSIGNMENT_DELAY DIB[((k+1)*CHUNKSIZE)-1 : k*CHUNKSIZE];
                  DOB_R[((k+1)*CHUNKSIZE)-1 : k*CHUNKSIZE]      <= `BSV_ASSIGNMENT_DELAY DIB[((k+1)*CHUNKSIZE)-1 : k*CHUNKSIZE];
               end
               else begin
                  DOB_R[((k+1)*CHUNKSIZE)-1 : k*CHUNKSIZE]      <= `BSV_ASSIGNMENT_DELAY RAM[ADDRB][((k+1)*CHUNKSIZE)-1 : k*CHUNKSIZE];
               end
            end
         end
      end
   endgenerate

   // Output drivers
   always @(posedge CLKA) begin
      DOA_R2 <= `BSV_ASSIGNMENT_DELAY DOA_R;
   end

   always @(posedge CLKB) begin
      DOB_R2 <= `BSV_ASSIGNMENT_DELAY DOB_R;
   end

   assign DOA = (PIPELINED) ? DOA_R2 : DOA_R;
   assign DOB = (PIPELINED) ? DOB_R2 : DOB_R;

endmodule // BRAM2BELoad
`endif
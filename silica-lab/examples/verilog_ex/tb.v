`timescale 1ns/1ps

module tb;
    reg clk;
    reg rst;
    wire q;

    // Instantiate the top module
    top uut (
        .clk(clk),
        .rst(rst),
        .q(q)
    );

    // Clock generation: 10ns period
    initial clk = 0;
    always #5 clk = ~clk;

    // Test sequence
    initial begin
        $dumpfile("build/wave.vcd");
        $dumpvars(0, tb);

        rst = 1;
        #10;
        rst = 0;

        #100; // Run for 100ns
        $finish;
    end
endmodule

`timescale 1ns/1ps

module tb;
    logic clk;
    logic rst;
    logic q;

    // Instantiate top
    top uut (
        .clk(clk),
        .rst(rst),
        .q(q)
    );

    // Clock generation
    initial clk = 0;
    always #5 clk = ~clk;

    // Test sequence
    initial begin
        $dumpfile("build/wave.vcd");
        $dumpvars(0, tb);

        rst = 1;
        #10;
        rst = 0;

        #100;
        $finish;
    end
endmodule

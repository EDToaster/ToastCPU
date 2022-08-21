interface io_interface;
    logic clock;
    
    logic [15:0] waddr, raddr, wdata, rdata;
    logic wenable;
    
    // irq
    logic irq;
    logic reset_irq;
endinterface
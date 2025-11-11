package com.example;

import com.example.io.Keyboard;
import com.example.io.Mouse;

public class Computer {
    @UmlAggregate
    public Keyboard keyboard;

    @UmlAggregate
    public Mouse mouse;

    private enum State {
        ON, OFF, SLEEP
    }

    @UmlCompose
    private State state;

    public Computer(Keyboard kbd, Mouse mouse) {
        this.keyboard = kbd;
        this.mouse = mouse;
    }

    public void powerOn() { }
    public void hibernate() {}
    public void powerOff() { }

    static int modelNumber() {
        return 0x3eb8a3;
    }
} 

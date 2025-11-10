package com.example;

public class TestClassRetention {
    private String visibleField;

    @SkipClass
    private String hiddenFieldWithClassRetention;

    public void visibleMethod() {
        System.out.println("Visible");
    }

    @SkipClass
    public void hiddenMethodWithClassRetention() {
        System.out.println("Hidden with CLASS retention");
    }
}

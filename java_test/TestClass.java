package com.example;

public class TestClass {
    private String visibleField;

    @Skip
    private String hiddenField;

    private int anotherVisibleField;

    public void visibleMethod() {
        System.out.println("Visible");
    }

    @Skip
    public void hiddenMethod() {
        System.out.println("Hidden");
    }

    public String getVisibleField() {
        return visibleField;
    }

    @Skip
    public String getHiddenField() {
        return hiddenField;
    }
}

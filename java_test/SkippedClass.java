package com.example;

@Skip
public class SkippedClass {
    private String someField;

    public void someMethod() {
        System.out.println("This whole class should be skipped");
    }
}

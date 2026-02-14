function example(input: unknown): number {
    assertType<string>(input);
    return input.length;
}

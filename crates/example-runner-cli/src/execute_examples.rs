trait RunnableExample {}

struct ExamplesExecutor {
    examples: Vec<Box<dyn RunnableExample>>,
}

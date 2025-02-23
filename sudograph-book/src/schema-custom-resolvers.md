# Custom resolvers

Custom resolvers where the resolver function is defined within the same Rust canister as the `graphql_database` macro are possible now, but not well-documented.  See the [intermediate example](https://github.com/sudograph/sudograph/tree/main/examples/intermediate) for a very rough overview of how to achieve this.

Custom resolvers where the resolver function is defined in a separate canister (be it written in Rust, Motoko, AssemblyScript, or any other language) are also supported. For now you will need to write a custom resolver in Rust that does a cross-canister call to the resolver in the separate canister. This is not well-documented but is possible. In the future, the plan is for those types of custom resolvers to be defined only in the schema so that no Rust code is required. It will look like this:

```graphql
# schema.graphql

type Query {
    myCustomQueryResolver(param1: Int!): Boolean! @canister(id: "renrk-eyaaa-aaaaa-aaada-cai")
}

type Mutation {
    myCustomMutationResolver(param1: Float!): String! @canister(id: "rdmx6-jaaaa-aaaaa-aaadq-cai")
}
```

Defining the above `Query` and `Mutation` object types in your schema would generate two custom resolver functions in the Rust canister where the `graphql_database` macro is used. These resolver functions would perform cross-canister calls to a canister with the id defined in the `@canister` directive. You would then need to define `myCustomQueryResolver` with the appropriate parameters and return type in the `renrk-eyaaa-aaaaa-aaada-cai` canister, and `myCustomMutationResolver` with the appropriate parameters and return type in the `rdmx6-jaaaa-aaaaa-aaadq-cai` canister.
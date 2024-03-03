# elna_RAG

Canister to manage orchestrates the prompt engine and linking it with the vector database for ELNA project


## Running the project locally

```bash
dfx start --background
dfx deploy
```

Once the job completes, your application will be available at `http://localhost:4943?canisterId={canister_id}`.

If you have made changes to exposed candid methods, you can generate a new candid interface with

```bash
./did.sh
```
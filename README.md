# moooo.farm

Moooooonitoring the cow.txt herd.

## API

Please see [https://mooo.farm/api](https://mooo.farm/api) for the list of all endpoints.

## Local development

1. Make sure that you have `rustup` installed and you are using the Rust toolchain via `rustup`. It should automatically
   pick up on the fact that you need the `nightly` toolchain when running `cargo` in the context of this project in the
   next steps.
2. Enter the backend directory:

       $ cd moooodotfarm-backend

3. Run the application locally:

       $ make local

4. Open your browser and navigate to [http://localhost:8080](http://localhost:8080) to see the application running.

5. The same checks (tests, linter) that are executed to validate your pull request can be locally run with:

       $ make ci

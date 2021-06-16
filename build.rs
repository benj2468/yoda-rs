use schema::YodaSchema;
use std::fs::File;
use std::io::prelude::*;

// Example custom build script.
fn main() {
    let schema =
        YodaSchema::build(Default::default(), Default::default(), Default::default()).finish();

    let sdl = schema.sdl();

    let mut file = File::create("graphql/schema.graphql").expect("Could not create file");

    file.write(sdl.as_bytes())
        .expect("Could not write the file?");
}

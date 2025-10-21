//!
//! This example shows features requiring the 'web' feature to work
//! Stuff like setTimeout, atob/btoa, file reads and fetch are all examples
//!
//! We will focus on timers and fetch here
use std::time::Duration;

use rustyscript::{json_args, Error, Module, Runtime, RuntimeOptions};

fn main() -> Result<(), Error> {
    // This module has an async function, which is not itself a problem
    // However, it uses setTimeout - the timer will never be triggered
    // unless the web feature is active.
    // See above for a longer list for web feature exclusives
    let module = Module::new(
        "test.js",
        "
        const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
        export async function test() {
            await sleep(10);
            return 2;
        }

        export async function fetch_example() {
            return new Promise((accept, reject) => {
                fetch('https://api.github.com/users/mralexgray/repos', {
                    method: 'GET',
                    headers: {
                      Accept: 'application/json',
                    },
                  }).then(response => response.json())
                  .then(json => accept(json))
                  .catch(e => reject(e));
            });
        }

        export async function event_source_example() {
            return new Promise((accept, reject) => {
                var source = new EventSource('https://www.w3schools.com/html/demo_sse.php');
                source.addEventListener('message', (e) => {
                    accept(e.data);
                });
                source.addEventListener('error', (e) => {
                  reject(e)
                });
            });
        }
        ",
    );

    // We add a timeout to the runtime anytime async might be used
    let mut runtime = Runtime::new(RuntimeOptions {
        timeout: Duration::from_millis(10000),
        ..Default::default()
    })?;

    // The async function
    let module_handle = runtime.load_module(&module)?;
    let value: usize = runtime.call_function(Some(&module_handle), "test", json_args!())?;
    println!("Got value: {value}");
    assert_eq!(value, 2);

    // Fetch example
    let data: rustyscript::serde_json::Value =
        runtime.call_function(Some(&module_handle), "fetch_example", json_args!())?;
    println!("Got {:?} bytes", data.to_string().len());

    // EventSource example
    let data: rustyscript::serde_json::Value =
        runtime.call_function(Some(&module_handle), "event_source_example", json_args!())?;
    println!("Got event: {data}");

    Ok(())
}

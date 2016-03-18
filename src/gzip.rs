extern crate flate2;

//
// Need 'self' here because an 'extern crate' statement loads
// the variable into the current namespace while a 'use' statement
// is an absolute path. An alternative is to actually specify the
// full path, e.g.,
// 'use gzip::flate2::write::GzEncoder'
// 
// 'self' is more flexible since it doesn't need to be changed should
// the name of the module in which this code is being used change.
//
use self::flate2::write::GzEncoder;
use self::flate2::Compression::Default;

// Rust Note
// It's necessary for the 'Write' trait to be in scope
// because 'write_all' is a 'Write' trait method. Details
// from the compiler:
// "items from traits can only be used if the trait is in
//  scope; the following trait 'write_all' is implemented
//  but not in scope, perhaps add a `use` for it:
//  'use std::io::Write'"

use std::io::Write;

#[test]
fn test_encode() {
  let mut input = Vec::new();
  input.extend_from_slice(&[49,49,49,49,49,50,50,50,50,50,51,51,51,51,51,52,52,52,52,52]);
  let mut expected = Vec::new();
  expected.extend_from_slice(&[31,139,8,0,0,0,0,0,0,7,51,52,4,2,35,16,48,6,1,19,16,0,0,103,130,220,216,20,0,0,0]);

  let compressed = encode(input);

  assert_eq!(compressed, expected);
}
 
pub fn encode(input: Vec<u8>) -> Vec<u8> {

  // 'write_stream' is an EncoderWriter<W> where 'W' is
  // the Write trait. The first argument to 'GzEncoder::new' is bound to
  // 'W', that is, it is bound to the Write trait. This is fine because
  // std::Vec implementes the Write trait.
  let mut write_stream = GzEncoder::new(Vec::new(), Default);

  // TODO: How can 'write_all', which is a Write method, implemented by
  // std::Vec, be invoked on 'write_stream', which is of type EncoderWriter<Write>?
  // EncoderWriter does not implement 'write_all'. EncoderWriter has an 'inner'
  // object that is a std::Vec but 'inner' is not exposed. I don't get it.
  write_stream.write_all(&input).unwrap();

  let bytes = match write_stream.finish() {
    Ok(response) => response,
    Err(_) => panic!("Error with 'finish'")
  };

  bytes 
}


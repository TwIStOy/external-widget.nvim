use all_asserts::*;
use ext_widget_core::nvim::find_file_in_runtime_path;
use nvim_oxi as oxi;

// #[oxi::test]
// fn test_find_parser() {
//     // add runtime path
//     let p = find_file_in_runtime_path("parser/lua.so");
//     assert_true!(p.is_err());
//     assert_true!(p.unwrap().is_some());
// }

#[oxi::test]
fn set_get_del_var() {
    oxi::api::set_var("foo", 42).unwrap();
    assert_eq!(Ok(42), oxi::api::get_var("foo"));
    assert_eq!(Ok(()), oxi::api::del_var("foo"));
}

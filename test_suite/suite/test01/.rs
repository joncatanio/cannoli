extern crate cannolib;
fn main() {
    let mut cannoli_scope_list: Vec<std::collections::HashMap<String, cannolib::Value>> = Vec::new();
    cannoli_scope_list.push(cannolib::builtin::get_scope());
    cannoli_scope_list.push(std::collections::HashMap::new());
    cannoli_scope_list.last_mut().unwrap().insert("__name__".to_string(), cannolib::Value::Str("__main__".to_string()));
    let mut cannoli_object_tbl = std::collections::HashMap::new();
    let mut v0 = cannolib::Value::Number(cannolib::NumericType::Integer(4));
    cannoli_object_tbl.insert("y".to_string(), v0);
    cannoli_object_tbl.insert("func".to_string(), cannolib::Value::Function { f: |mut cannoli_scope_list: Vec<std::collections::HashMap<String, cannolib::Value>>, cannoli_func_args: Vec<cannolib::Value>| -> cannolib::Value {
        cannoli_scope_list.push(std::collections::HashMap::new());
        let mut cannoli_func_args_iter = cannoli_func_args.into_iter();
        cannoli_scope_list.last_mut().unwrap().insert("self".to_string(), cannoli_func_args_iter.next().expect("expected 2 positional args"));
        cannoli_scope_list.last_mut().unwrap().insert("g".to_string(), cannoli_func_args_iter.next().expect("expected 2 positional args"));
        let mut v2 = cannolib::lookup_value(&cannoli_scope_list, "print");
        let mut v3 = cannolib::lookup_value(&cannoli_scope_list, "g");
        let mut v1 = v2.call(cannoli_scope_list.clone(), vec![v3]);
        cannolib::Value::None
    }});
    cannoli_object_tbl.insert("__class__".to_string(), cannolib::Value::Str("SomeClass".to_string()));
    cannoli_scope_list.last_mut().unwrap().insert("SomeClass".to_string(), cannolib::Value::Class { tbl: cannoli_object_tbl });
    let mut v5 = cannolib::lookup_value(&cannoli_scope_list, "SomeClass");
    let mut v4 = v5.call(cannoli_scope_list.clone(), vec![]);
    cannoli_scope_list.last_mut().unwrap().insert("c".to_string(), v4);
    let mut v7 = cannolib::lookup_value(&cannoli_scope_list, "c");
    let mut v8 = cannolib::Value::Str("member function call on class".to_string());
    let mut v6 = cannolib::call_member(v7, "func", cannoli_scope_list.clone(), vec![v8]);
}

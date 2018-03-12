extern crate cannolib;
pub mod main {
    use cannolib;
    use std;
    pub fn execute() {
        let mut cannoli_scope_list: Vec<std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, cannolib::Value>>>> = Vec::new();
        cannoli_scope_list.push(std::rc::Rc::new(std::cell::RefCell::new(cannolib::builtin::get_scope())));
        cannoli_scope_list.push(std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashMap::new())));
        cannoli_scope_list.last_mut().unwrap().borrow_mut().insert("__name__".to_string(), cannolib::Value::Str("__main__".to_string()));
        let mut cannoli_object_tbl = std::collections::HashMap::new();
        let mut v0 = cannolib::Value::Number(cannolib::NumericType::Integer(4));
        cannoli_object_tbl.insert("y".to_string(), v0);
        let move_scope = cannoli_scope_list.clone();
        let mut v1 = cannolib::Value::Function(std::rc::Rc::new(move |cannoli_func_args: Vec<cannolib::Value>| -> cannolib::Value {
            let mut cannoli_scope_list = move_scope.clone();
            cannoli_scope_list.push(std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashMap::new())));
            let mut cannoli_func_args_iter = cannoli_func_args.into_iter();
            cannoli_scope_list.last_mut().unwrap().borrow_mut().insert("self".to_string(), cannoli_func_args_iter.next().expect("expected 2 positional args"));
            cannoli_scope_list.last_mut().unwrap().borrow_mut().insert("g".to_string(), cannoli_func_args_iter.next().expect("expected 2 positional args"));
            let mut v3 = cannolib::lookup_value(&cannoli_scope_list, "print");
            let mut v4 = cannolib::lookup_value(&cannoli_scope_list, "g");
            let mut v2 = v3.call(vec![v4]);
            cannolib::Value::None
        }));
        cannoli_object_tbl.insert("func".to_string(), v1);
        cannoli_object_tbl.insert("__class__".to_string(), cannolib::Value::Str("SomeClass".to_string()));
        cannoli_scope_list.last_mut().unwrap().borrow_mut().insert("SomeClass".to_string(), cannolib::Value::Class { tbl: cannoli_object_tbl });
        let mut v6 = cannolib::lookup_value(&cannoli_scope_list, "SomeClass");
        let mut v5 = v6.call(vec![]);
        cannoli_scope_list.last_mut().unwrap().borrow_mut().insert("c".to_string(), v5);
        let mut v8 = cannolib::lookup_value(&cannoli_scope_list, "c");
        let mut v9 = cannolib::Value::Str("member function call on class".to_string());
        let mut v7 = cannolib::call_member(v8, "func", vec![v9]);
    }
}

extern crate cannolib;
pub mod main {
    use cannolib;
    use std;
    pub fn execute() {
        let mut cannoli_scope_list: Vec<std::rc::Rc<std::cell::RefCell<std::collections::HashMap<String, cannolib::Value>>>> = Vec::new();
        cannoli_scope_list.push(std::rc::Rc::new(std::cell::RefCell::new(cannolib::builtin::get_scope())));
        cannoli_scope_list.push(std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashMap::new())));
        cannoli_scope_list.last_mut().unwrap().borrow_mut().insert("__name__".to_string(), cannolib::Value::Str("__main__".to_string()));
        let mut v2 = cannolib::Value::Bool(true);
        let mut v3 = cannolib::Value::Bool(false);
        let mut v1 = cannolib::Value::Bool((v2).to_bool() && (v3).to_bool());
        let mut v5 = cannolib::Value::Bool(true);
        let mut v6 = cannolib::Value::Bool(false);
        let mut v7 = cannolib::Value::Bool(true);
        let mut v4 = cannolib::Value::Bool((v5).to_bool() && (v6).to_bool() && (v7).to_bool());
        let mut v8 = cannolib::Value::Bool(true);
        let mut v0 = cannolib::Value::Bool((v1).to_bool() || (v4).to_bool() || (v8).to_bool());
        if (v0).to_bool() {
            let mut v10 = cannolib::lookup_value(&cannoli_scope_list, "print");
            let mut v11 = cannolib::Value::Str("hello".to_string());
            let mut v9 = v10.call(vec![v11]);
            let mut v12 = cannolib::Value::Number(cannolib::NumericType::Integer(5));
            cannoli_scope_list.last_mut().unwrap().borrow_mut().insert("x".to_string(), v12);
            let mut v15 = cannolib::lookup_value(&cannoli_scope_list, "x");
            let mut v16 = cannolib::Value::Number(cannolib::NumericType::Integer(12));
            let mut v14 = cannolib::Value::Bool((v15 > v16));
            let mut v18 = cannolib::lookup_value(&cannoli_scope_list, "x");
            let mut v19 = cannolib::Value::Number(cannolib::NumericType::Integer(9));
            let mut v17 = cannolib::Value::Bool((v18 < v19));
            let mut v13 = cannolib::Value::Bool((v14).to_bool() || (v17).to_bool());
            if (v13).to_bool() {
                let mut v21 = cannolib::lookup_value(&cannoli_scope_list, "print");
                let mut v22 = cannolib::Value::Str("it is!".to_string());
                let mut v20 = v21.call(vec![v22]);
            }
        }
    }
}

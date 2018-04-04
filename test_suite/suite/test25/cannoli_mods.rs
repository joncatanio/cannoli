extern crate cannolib;
pub mod main {
    use cannolib;
    use std;
    pub fn execute() {
        let mut cannoli_scope_list: Vec<std::rc::Rc<std::cell::RefCell<Vec<cannolib::Value>>>> = Vec::new();
        cannoli_scope_list.push(std::rc::Rc::new(std::cell::RefCell::new(cannolib::builtin::get_scope())));
        cannoli_scope_list.push(std::rc::Rc::new(std::cell::RefCell::new(std::collections::HashMap::new())));
        cannoli_scope_list.last_mut().unwrap().borrow_mut().insert("__name__".to_string(), cannolib::Value::Str("__main__".to_string()));
        let mut v0 = cannolib::Value::Number(cannolib::NumericType::Integer(3));
        cannoli_scope_list.last_mut().unwrap().borrow_mut().insert("a".to_string(), v0);
        let mut v1 = cannolib::Value::Str("hello".to_string());
        cannoli_scope_list.last_mut().unwrap().borrow_mut().insert("b".to_string(), v1);
        let mut v3 = cannolib::lookup_value(&cannoli_scope_list, "print");
        let mut v4 = cannolib::lookup_value(&cannoli_scope_list, "a");
        let mut v5 = cannolib::lookup_value(&cannoli_scope_list, "b");
        let mut kwargs = std::collections::HashMap::new();
        let mut v2 = v3.call(vec![v4, v5], kwargs);
    }
}

extern crate cannolib;
pub mod main {
    use cannolib;
    use std;
    pub fn execute() {
        let mut cannoli_scope_list: Vec<std::rc::Rc<std::cell::RefCell<Vec<cannolib::Value>>>> = Vec::new();
        cannoli_scope_list.push(std::rc::Rc::new(std::cell::RefCell::new(cannolib::builtin::get_scope())));
        let mut scope_list_setup: Vec<cannolib::Value> = Vec::with_capacity(3);
        scope_list_setup.resize(3, cannolib::Value::Undefined);
        cannoli_scope_list.push(std::rc::Rc::new(std::cell::RefCell::new(scope_list_setup)));
        cannoli_scope_list[1].borrow_mut()[2] = cannolib::Value::Str("__main__".to_string());
        let mut cannoli_object_tbl = std::collections::HashMap::new();
        let mut v0 = cannolib::Value::Number(cannolib::NumericType::Integer(5));
        cannoli_object_tbl.insert("x".to_string(), v0);
        let mut v1 = cannolib::Value::Number(cannolib::NumericType::Integer(2));
        cannoli_object_tbl.insert("y".to_string(), v1);
        let move_scope = cannoli_scope_list.clone();
        let mut v2 = cannolib::Value::Function(std::rc::Rc::new(move |cannoli_func_args: Vec<cannolib::Value>, mut kwargs: std::collections::HashMap<String, cannolib::Value>| -> cannolib::Value {
            let mut cannoli_scope_list = move_scope.clone();
            let mut scope_list_setup: Vec<cannolib::Value> = Vec::with_capacity(3);
            scope_list_setup.resize(3, cannolib::Value::Undefined);
            cannoli_scope_list.push(std::rc::Rc::new(std::cell::RefCell::new(scope_list_setup)));
            let mut cannoli_func_args_iter = cannoli_func_args.into_iter();
            cannoli_scope_list[2].borrow_mut()[0] = cannoli_func_args_iter.next().unwrap_or(cannolib::Value::None); // param_name: 'self'
            cannoli_scope_list[2].borrow_mut()[2] = cannoli_func_args_iter.next().unwrap_or(cannolib::Value::None); // param_name: 'y'
            cannoli_scope_list[2].borrow_mut()[1] = cannoli_func_args_iter.next().unwrap_or(cannolib::Value::None); // param_name: 'z'
            let mut v3 = cannoli_scope_list[2].borrow()[2].clone();
            let mut v4 = cannoli_scope_list[2].borrow()[0].clone();
            cannolib::attr_assign(v4, "first", v3);
            let mut v5 = cannoli_scope_list[2].borrow()[1].clone();
            let mut v6 = cannoli_scope_list[2].borrow()[0].clone();
            cannolib::attr_assign(v6, "second", v5);
            let mut v7 = cannolib::Value::Bool(true);
            if (v7).to_bool() {
                let mut v8 = cannolib::Value::Bool(true);
                let mut v9 = cannoli_scope_list[2].borrow()[0].clone();
                cannolib::attr_assign(v9, "true", v8);
            } else {
                let mut v10 = cannolib::Value::Bool(false);
                let mut v11 = cannoli_scope_list[2].borrow()[0].clone();
                cannolib::attr_assign(v11, "true", v10);
                let mut v12 = cannolib::Value::Bool(true);
                let mut v13 = cannoli_scope_list[2].borrow()[0].clone();
                cannolib::attr_assign(v13, "false", v12);
            }
            let mut v15 = cannolib::Value::Number(cannolib::NumericType::Integer(500));
            let mut v16 = cannolib::Value::Number(cannolib::NumericType::Integer(600));
            let mut cannoli_tuple_builder = Vec::new();
            cannoli_tuple_builder.push(v15);
            cannoli_tuple_builder.push(v16);
            let mut v14 = cannolib::Value::Tuple(cannolib::TupleType::new(cannoli_tuple_builder));
            let mut v17 = v14.index(cannolib::Value::Number(cannolib::NumericType::Integer(0)));
            let mut v18 = cannoli_scope_list[2].borrow()[0].clone();
            cannolib::attr_assign(v18, "tup1", v17);
            let mut v19 = v14.index(cannolib::Value::Number(cannolib::NumericType::Integer(1)));
            let mut v20 = cannoli_scope_list[2].borrow()[0].clone();
            cannolib::attr_assign(v20, "tup2", v19);
            cannolib::Value::None
        }));
        cannoli_object_tbl.insert("__init__".to_string(), v2);
        let move_scope = cannoli_scope_list.clone();
        let mut v21 = cannolib::Value::Function(std::rc::Rc::new(move |cannoli_func_args: Vec<cannolib::Value>, mut kwargs: std::collections::HashMap<String, cannolib::Value>| -> cannolib::Value {
            let mut cannoli_scope_list = move_scope.clone();
            let mut scope_list_setup: Vec<cannolib::Value> = Vec::with_capacity(4);
            scope_list_setup.resize(4, cannolib::Value::Undefined);
            cannoli_scope_list.push(std::rc::Rc::new(std::cell::RefCell::new(scope_list_setup)));
            let mut cannoli_func_args_iter = cannoli_func_args.into_iter();
            cannoli_scope_list[2].borrow_mut()[2] = cannoli_func_args_iter.next().unwrap_or(cannolib::Value::None); // param_name: 'self'
            cannoli_scope_list[2].borrow_mut()[3] = cannoli_func_args_iter.next().unwrap_or(cannolib::Value::None); // param_name: 'a'
            cannoli_scope_list[2].borrow_mut()[0] = cannoli_func_args_iter.next().unwrap_or(cannolib::Value::None); // param_name: 'b'
            cannoli_scope_list[2].borrow_mut()[1] = cannoli_func_args_iter.next().unwrap_or(cannolib::Value::None); // param_name: 'c'
            let mut v23 = cannoli_scope_list[0].borrow()[0].clone();
            let mut v24 = cannoli_scope_list[2].borrow()[3].clone();
            let mut v25 = cannoli_scope_list[2].borrow()[0].clone();
            let mut v26 = cannoli_scope_list[2].borrow()[1].clone();
            let mut kwargs = std::collections::HashMap::new();
            let mut v22 = v23.call(vec![v24, v25, v26], kwargs);
            cannolib::Value::None
        }));
        cannoli_object_tbl.insert("some_func".to_string(), v21);
        let move_scope = cannoli_scope_list.clone();
        let mut v27 = cannolib::Value::Function(std::rc::Rc::new(move |cannoli_func_args: Vec<cannolib::Value>, mut kwargs: std::collections::HashMap<String, cannolib::Value>| -> cannolib::Value {
            let mut cannoli_scope_list = move_scope.clone();
            let mut scope_list_setup: Vec<cannolib::Value> = Vec::with_capacity(1);
            scope_list_setup.resize(1, cannolib::Value::Undefined);
            cannoli_scope_list.push(std::rc::Rc::new(std::cell::RefCell::new(scope_list_setup)));
            let mut cannoli_func_args_iter = cannoli_func_args.into_iter();
            cannoli_scope_list[2].borrow_mut()[0] = cannoli_func_args_iter.next().unwrap_or(cannolib::Value::None); // param_name: 'b'
            let mut v29 = cannoli_scope_list[0].borrow()[0].clone();
            let mut v30 = cannolib::Value::Str("Class func".to_string());
            let mut v31 = cannoli_scope_list[2].borrow()[0].clone();
            let mut kwargs = std::collections::HashMap::new();
            let mut v28 = v29.call(vec![v30, v31], kwargs);
            cannolib::Value::None
        }));
        cannoli_object_tbl.insert("class_func".to_string(), v27);
        cannoli_object_tbl.insert("__name__".to_string(), cannolib::Value::Str("Test".to_string()));
        cannoli_scope_list[1].borrow_mut()[1] = cannolib::Value::Class { tbl: cannoli_object_tbl }; // class: 'Test'
        let mut v33 = cannoli_scope_list[1].borrow()[1].clone();
        let mut v34 = cannolib::Value::Str("first".to_string());
        let mut v35 = cannolib::Value::Str("second".to_string());
        let mut kwargs = std::collections::HashMap::new();
        let mut v32 = v33.call(vec![v34, v35], kwargs);
        cannoli_scope_list[1].borrow_mut()[0] = v32; // id: 'a'
        let mut v37 = cannoli_scope_list[0].borrow()[0].clone();
        let mut v39 = cannoli_scope_list[1].borrow()[0].clone();
        let mut v38 = v39.get_attr("x");
        let mut v41 = cannoli_scope_list[1].borrow()[0].clone();
        let mut v40 = v41.get_attr("y");
        let mut v43 = cannoli_scope_list[1].borrow()[0].clone();
        let mut v42 = v43.get_attr("first");
        let mut v45 = cannoli_scope_list[1].borrow()[0].clone();
        let mut v44 = v45.get_attr("second");
        let mut v47 = cannoli_scope_list[1].borrow()[0].clone();
        let mut v46 = v47.get_attr("true");
        let mut v49 = cannoli_scope_list[1].borrow()[0].clone();
        let mut v48 = v49.get_attr("tup1");
        let mut v51 = cannoli_scope_list[1].borrow()[0].clone();
        let mut v50 = v51.get_attr("tup2");
        let mut kwargs = std::collections::HashMap::new();
        let mut v36 = v37.call(vec![v38, v40, v42, v44, v46, v48, v50], kwargs);
        let mut v53 = cannoli_scope_list[1].borrow()[0].clone();
        let mut v54 = cannolib::Value::Str("a".to_string());
        let mut v55 = cannolib::Value::Str("b".to_string());
        let mut v56 = cannolib::Value::Str("c".to_string());
        let mut kwargs = std::collections::HashMap::new();
        let mut v52 = cannolib::call_member(v53, "some_func", vec![v54, v55, v56], kwargs);
        let mut v58 = cannoli_scope_list[1].borrow()[1].clone();
        let mut v59 = cannolib::Value::Str("woo".to_string());
        let mut kwargs = std::collections::HashMap::new();
        let mut v57 = cannolib::call_member(v58, "class_func", vec![v59], kwargs);
    }
}

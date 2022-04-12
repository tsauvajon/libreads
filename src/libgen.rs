use pyo3::prelude::*;

const LIBGEN_API_MODULE_NAME: &str = "libgen_api";
const LIBGEN_SEARCH_CLASS_NAME: &str = "LibgenSearch";
const SEARCH_TITLE_METHOD_NAME: &str = "search_title";

pub fn get_metadata(isbn: &str) -> PyResult<()> {
    Python::with_gil(|py| {
        let libgen_api_module = py.import(LIBGEN_API_MODULE_NAME).unwrap();
        let libgen_search_class = libgen_api_module.getattr(LIBGEN_SEARCH_CLASS_NAME).unwrap();
        let libgen_search = libgen_search_class.call0().unwrap();

        let results = libgen_search
            .call_method1(SEARCH_TITLE_METHOD_NAME, (isbn,))
            .unwrap();

        // TODO:
        // - find the most relevant result
        // - call libgen_search.resolve_download_links(_) on it

        println!("results: {:?}", results);

        // let house = libgen_search_class.call1(("123 Main Street",)).unwrap();

        // house.call_method0("__enter__").unwrap();

        // let result = py.eval("undefined_variable + 1", None, None);

        // // If the eval threw an exception we'll pass it through to the context manager.
        // // Otherwise, __exit__  is called with empty arguments (Python "None").
        // match result {
        //     Ok(_) => {
        //         let none = py.None();
        //         house
        //             .call_method1("__exit__", (&none, &none, &none))
        //             .unwrap();
        //     }
        //     Err(e) => {
        //         house
        //             .call_method1("__exit__", (e.get_type(py), e.value(py), e.traceback(py)))
        //             .unwrap();
        //     }
        // }
    });

    Ok(())
}

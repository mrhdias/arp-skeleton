//
// Shared library skeleton
//

use std::ffi::{
    c_char,
    CStr,
    CString,
};
use serde::{Deserialize, Serialize};
use hyper::HeaderMap;

static VERSION: &'static str = "0.1.0";

// mandatory struct
#[derive(Debug, Serialize)]
struct PluginRoute {
    path: &'static str,
    function: &'static str,
    method_router: &'static str,
    response_type: &'static str,
}

// add here all available routes for this plugin
static ROUTES: &[PluginRoute] = &[
    PluginRoute {
        path: "/groceries",
        function: "groceries",
        method_router: "post",
        response_type: "html",
    },
    PluginRoute {
        path: "/products",
        function: "products_get",
        method_router: "get",
        response_type: "json",
    },
    PluginRoute {
        path: "/products",
        function: "products_post",
        method_router: "post",
        response_type: "json",
    },
    PluginRoute {
        path: "/about",
        function: "about",
        method_router: "get",
        response_type: "text",
    },
];

// --- functions ---

#[derive(Clone, Deserialize, Serialize)]
struct Product {
    id: Option<u32>,
    name: String,
    image_url: String,
    price: f64,
}

fn products() -> Vec<Product> {
    vec![
        Product {
            id: Some(1),
            name: "Lorem ipsum dolor".to_string(),
            image_url: "https://picsum.photos/210/300".to_string(),
            price: 39.99,
        },
        Product {
            id: Some(2),
            name: "Donec rutrum dui".to_string(),
            image_url: "https://picsum.photos/220/300".to_string(),
            price: 59.99,
        },
        Product {
            id: Some(3),
            name: "Mauris imperdiet massa".to_string(),
            image_url: "https://picsum.photos/230/300".to_string(),
            price: 29.99,
        },
        Product {
            id: Some(4),
            name: "Sed tristique tellus".to_string(),
            image_url: "https://picsum.photos/240/300".to_string(),
            price: 9.99,
        },
        Product {
            id: Some(5),
            name: "Vivamus tempus".to_string(),
            image_url: "https://picsum.photos/250/300".to_string(),
            price: 49.99,
        },
        Product {
            id: Some(6),
            name: "Aliquam rutrum viverra".to_string(),
            image_url: "https://picsum.photos/260/300".to_string(),
            price: 19.99,
        },
    ]
}

#[no_mangle]
pub extern "C" fn products_post(
    headers: *mut HeaderMap,
    body: *const c_char,
) -> *const c_char {

    if headers.is_null() || body.is_null() {
        // Handle the null pointer case
        return std::ptr::null_mut();
    }

    // Convert headers pointer to a reference
    let headers = unsafe { &*headers };

    println!("Headers: {:?}", headers);

    match headers.get("content-type") {
        Some(value) => {
            if value.to_str().unwrap_or("").to_string() != "application/json" {
                let c_response = CString::new(format!(r#"{{"error": "Invalid content type: {:?}"}}"#, value))
                    .unwrap();
                return c_response.into_raw();
            }
        },
        None => {
            let c_response = CString::new(r#"{"error": "No content type"}"#)
                .unwrap();
            return c_response.into_raw();
        },
    };

    // Convert body pointer to a Rust string
    let body_str = unsafe {
        CStr::from_ptr(body)
            .to_str()
            .unwrap_or("Invalid UTF-8 sequence") // Handle possible UTF-8 errors
    };

    println!("Body Str: {}", body_str);

    let mut products = products();

    let mut product: Product = serde_json::from_str(body_str).unwrap();
    product.id = Some(products.len() as u32);

    products.push(product);

    let pretty_json = serde_json::to_string_pretty(&products).unwrap();

    let c_response = CString::new(pretty_json).unwrap();
    c_response.into_raw()
}

#[derive(Debug, Serialize, Deserialize)]
struct QueryParameters {
    limit: Option<i32>,
    orderby: Option<String>,
}

// Example: http://127.0.0.1:8080/plugin/arp-skeleton/products?limit=3&orderby=price
#[no_mangle]
pub extern "C" fn products_get(
    headers: *mut HeaderMap,
    body: *const c_char,
) -> *const c_char {

    if headers.is_null() || body.is_null() {
        // Handle the null pointer case
        return std::ptr::null_mut();
    }

    // Convert headers pointer to a reference
    let headers = unsafe { &*headers };

    println!("Headers: {:?}", headers);

    let query = match headers.get("x-raw-query") {
        Some(value) => value.to_str().unwrap_or("").to_string(),
        None => String::new(),
    };

    // Parse and decode the URL-encoded form data
    let parameters: QueryParameters = serde_urlencoded::from_str(&query).unwrap();

    println!("Decoded Query: {:?}", parameters);

    // Convert body pointer to a Rust string
    let body_str = unsafe {
        CStr::from_ptr(body)
            .to_str()
            .unwrap_or("Invalid UTF-8 sequence") // Handle possible UTF-8 errors
    };

    println!("Body Str: {}", body_str);

    let limit = match parameters.limit {
        Some(limit) => limit,
        None => 4,
    };

    let orderby = match parameters.orderby {
        Some(ref orderby) => orderby.as_str(),
        None => "id",
    };

    let mut products = products();

    match orderby {
        "name" => products.sort_by(|a, b| a.name.cmp(&b.name)),
        "price" => products.sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap()),
        _ => products.sort_by(|a, b| a.id.cmp(&b.id)),
    };

    // Convert the limit from i32 to usize
    let limit = std::cmp::min(limit as usize, products.len());
    let products_by_limit = products[0..limit].to_vec();

    // Convert the result to a JSON string
    let json_string = serde_json::to_string(&products_by_limit).unwrap();

    let c_response = CString::new(json_string).unwrap();
    c_response.into_raw()
}

// example
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Grocery {
    name: String,
    price: f64,
    location: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GroceriesList {
    products: Vec<Grocery>,
}
#[no_mangle]
pub extern "C" fn groceries(
    _headers: *mut HeaderMap,
    _body: *const c_char,
) -> *const c_char {

    let url = "https://raw.githubusercontent.com/mdn/dom-examples/main/fetch/fetch-json/products.json";
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(url)
        .send()
        .unwrap()
        .json::<GroceriesList>() // Deserialize response to `GroceriesList`
        .unwrap();
    
    println!("Response: {:?}", response);

    // let serialized = serde_json::to_string(&response).unwrap();

    let mut bag = vec![];
    for product in response.products {
        bag.push(format!(r#"<li>
    <strong>{}</strong>
    can be found in {}:
    <strong>£{}</strong>
</li>"#, product.name, product.location, product.price));
    }

    let html = format!(r#"<div class="groceries">
<h1>Fetch json example</h1>
<ul>{}</ul>
</div>"#, bag.join("\n"));

    let c_response = CString::new(html).unwrap();
    c_response.into_raw()
}

// mandatory function
#[no_mangle]
pub extern "C" fn routes() -> *const c_char {

    let json_routes = serde_json::to_string_pretty(ROUTES)
        .unwrap_or("[]".to_string());

    let c_response = CString::new(json_routes.as_str())
        .unwrap();
    c_response
        .into_raw()
}

#[no_mangle]
pub extern "C" fn about(
    _headers: *mut HeaderMap,
    _body: *const c_char,
) -> *const c_char {

    let info = format!(r#"Name: arp-skeleton
Version: {}
authors = "Henrique Dias <mrhdias@gmail.com>"
Description: Shared library skeleton
License: MIT"#, VERSION);

    let c_response = CString::new(info).unwrap();
    c_response.into_raw()
}

// mandatory function
#[no_mangle]
pub extern "C" fn free(ptr: *mut c_char) {
    if ptr.is_null() { // Avoid dereferencing null pointers
        return;
    }

    // Convert the raw pointer back to a CString and drop it to free the memory
    unsafe {
        drop(CString::from_raw(ptr)); // Takes ownership of the memory and frees it when dropped
    }
}
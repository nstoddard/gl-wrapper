use std::collections::*;

// TODO: see if these `cfg`s can be avoided/merged
#[cfg(target_arch = "wasm32")]
use futures::future::*;
#[cfg(target_arch = "wasm32")]
use js_sys::*;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use std::mem;
#[cfg(target_arch = "wasm32")]
use std::ops::*;
#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::*;

#[cfg(not(target_arch = "wasm32"))]
use std::fs::File;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Read;

/// Stores assets that have been loaded. Currently, a URL can be loaded as a `Vec<u8>` or
/// an `HtmlImageElement`/`DynamicImage` (depending on platform).
pub struct Assets {
    assets: HashMap<String, Vec<u8>>,
    #[cfg(target_arch = "wasm32")]
    images: HashMap<String, HtmlImageElement>,
    #[cfg(not(target_arch = "wasm32"))]
    images: HashMap<String, image::DynamicImage>,
}

impl Assets {
    // TODO: make this version async
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load(asset_paths: Vec<String>, image_paths: Vec<String>) -> Self {
        Self {
            assets: asset_paths
                .into_iter()
                .map(|asset_path| {
                    let mut asset = vec![];
                    File::open(&asset_path).unwrap().read_to_end(&mut asset).unwrap();
                    (asset_path, asset)
                })
                .collect(),
            images: image_paths
                .into_iter()
                .map(|image_path| {
                    let image = image::open(&image_path).unwrap();
                    (image_path, image)
                })
                .collect(),
        }
    }

    /// Asynchronously loads one or more assets from URLs.
    ///
    /// This can also load images, as `HtmlImageElement`s. It's also possible to load images
    /// as regular files using the `image` crate.
    ///
    /// This loads all assets concurrently. It's intended for large assets; small assets should
    /// usually be loaded at compile time with `include_str!` or `include_bytes!`.
    ///
    /// Panics if any asset can't be loaded.
    #[cfg(target_arch = "wasm32")]
    pub async fn load(asset_urls: Vec<String>, image_urls: Vec<String>) -> Self {
        let loaded_assets: Rc<RefCell<HashMap<String, Vec<u8>>>> =
            Rc::new(RefCell::new(Default::default()));
        let loaded_images: Rc<RefCell<HashMap<String, HtmlImageElement>>> =
            Rc::new(RefCell::new(Default::default()));

        let loaded_assets2 = loaded_assets.clone();
        let loaded_images2 = loaded_images.clone();

        let mut futures_to_block_on = vec![];

        for asset_url in asset_urls {
            let loaded_assets = loaded_assets.clone();
            let future = async move {
                let asset_url2 = asset_url.clone();

                let mut request_init = RequestInit::new();
                request_init.method("GET");
                request_init.mode(RequestMode::Cors);

                let request = Request::new_with_str_and_init(&asset_url, &request_init).unwrap();
                let request_promise = window().unwrap().fetch_with_request(&request);

                let response = JsFuture::from(request_promise).await.unwrap();
                let response: Response = response.dyn_into().unwrap();
                if !response.ok() {
                    panic!("Unable to load asset: {:?}", asset_url2);
                }
                let array_buffer = JsFuture::from(response.array_buffer().unwrap()).await.unwrap();
                let array_buffer: ArrayBuffer = array_buffer.into();
                let array: Uint8Array = Uint8Array::new(&array_buffer);
                let mut dst = vec![0; array_buffer.byte_length() as usize];
                array.copy_to(&mut dst);
                loaded_assets.borrow_mut().insert(asset_url.clone(), dst);
            };
            futures_to_block_on.push(Either::Left(future));
        }

        for image_url in image_urls {
            let loaded_images = loaded_images.clone();
            let future = async move {
                let image_element = window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .create_element("img")
                    .unwrap()
                    .dyn_into::<HtmlImageElement>()
                    .unwrap();

                let promise = Promise::new(&mut |resolve, _reject| {
                    let image_url2 = image_url.clone();
                    let image_url3 = image_url.clone();
                    let image_element2 = image_element.clone();
                    let loaded_images = loaded_images.clone();
                    let onload_handler = Rc::new(RefCell::new(None));
                    let onload_handler2 = onload_handler.clone();
                    *onload_handler.borrow_mut() = Some(Closure::wrap(Box::new(move || {
                        loaded_images
                            .borrow_mut()
                            .insert(image_url2.clone(), image_element2.clone());
                        resolve.call0(&resolve).unwrap();
                        onload_handler2.borrow_mut().take();
                    })
                        as Box<dyn FnMut()>));
                    image_element.set_onload(Some(
                        onload_handler.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
                    ));

                    let onerror_handler = Rc::new(RefCell::new(None));
                    let onerror_handler2 = onerror_handler.clone();
                    *onerror_handler.borrow_mut() = Some(Closure::wrap(Box::new(move || {
                        onerror_handler2.borrow_mut().take();
                        panic!("Unable to load image: {:?}", image_url3);
                        // TODO: reject here instead of panicking?
                    })
                        as Box<dyn FnMut()>));
                    image_element.set_onerror(Some(
                        onerror_handler.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
                    ));
                });

                image_element.set_src(&image_url);

                JsFuture::from(promise).await.unwrap();
            };
            futures_to_block_on.push(Either::Right(future));
        }

        join_all(futures_to_block_on).await;

        let assets: HashMap<String, Vec<u8>> =
            mem::replace(&mut loaded_assets2.borrow_mut(), Default::default());
        let images: HashMap<String, HtmlImageElement> =
            mem::replace(&mut loaded_images2.borrow_mut(), Default::default());
        Assets { assets, images }
    }

    /// Returns a reference to the given asset.
    pub fn get(&self, url: &str) -> Option<&[u8]> {
        self.assets.get(url).map(|x| x.as_slice())
    }

    /// Removes the given asset and returns it. If an asset is only needed in one place, this may
    /// reduce the number of required clones.
    pub fn remove(&mut self, url: &str) -> Option<Vec<u8>> {
        self.assets.remove(url)
    }

    /// Returns the given image.
    #[cfg(target_arch = "wasm32")]
    pub fn get_image(&self, url: &str) -> Option<&HtmlImageElement> {
        self.images.get(url)
    }

    /// Returns the given image.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_image(&self, url: &str) -> Option<&image::DynamicImage> {
        self.images.get(url)
    }

    /// Removes the given image and returns it. If an asset is only needed in one place, this may
    /// reduce the number of required clones.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn remove_image(&mut self, url: &str) -> Option<image::DynamicImage> {
        self.images.remove(url)
    }
}

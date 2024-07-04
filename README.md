# WASM Regression
A Rust-->WASM project for calculating a logistic regression in any CSV file. It includes boilerplate code to test in a browser.

## Table of Contents
- [Installation](#installation)
- [Usage](#usage)
- [Credits](#credits)
- [Support](#support) 

## Installation
Clone and install the dependencies and install wasm-pack with `cargo install wasm-pack`

Run `wasm-pack build --target web` to create the compiled WASM files, the binaries will be created in your `pkg` directory.

Copy the generated `pkg` folder to the `public` folder, so you can test it.

## Usage
You can test it with any web server, I recommend the [node http-server](https://www.npmjs.com/package/http-server) as it's very simple to use, you just need to run the `http-server` command in your `public` directory, but any web server from python to nginx will suffice.

You'll need a CSV file to test the app and know some machine learning basics, the target column values must be YES and NO for positive and negative values and your feature columns need to have values that can be casted to rust's f32, the str columns will be discarded automatically. I provide some datasets in the sets folder as examples.

## Credits
Diabetes and Kerala CSVs obtained from the [kgdata repository](https://github.com/danysc/kgdata)

How to calculate a ROC curve from [Mustafa Murat ARAT](https://mmuratarat.github.io/2019-10-01/how-to-compute-AUC-plot-ROC-by-hand)

Using CSS from [Simple.css Framework](https://simplecss.org/)

## Support
Please [open an issue](https://github.com/fraction/readme-boilerplate/issues/new) for support.
 
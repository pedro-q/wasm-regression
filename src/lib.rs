use linfa::Dataset;
use linfa::traits::{Fit, Predict};
use ndarray::ArrayBase;
use plotters::backend::SVGBackend;
use plotters::chart::{ChartBuilder, LabelAreaPosition};
use plotters::drawing::IntoDrawingArea; 
use plotters::series::LineSeries;
use plotters::style::{BLACK, WHITE};
use polars::{datatypes::{DataType, Float64Type}, prelude::{CsvReader, IndexOrder, SerReader}};
use serde::Deserialize;
use std::io::Cursor;
use wasm_bindgen::prelude::wasm_bindgen; 
use linfa_logistic::LogisticRegression; 
use base64::{engine::general_purpose, Engine as _};

// export the function to JavaScript
pub use wasm_bindgen_rayon::init_thread_pool; 

#[wasm_bindgen]
pub fn analyze_file(buffer: &[u8]) -> String {
    let mut output = String::new();

    let cursor = Cursor::new(buffer);
    let dataframe = CsvReader::new(cursor)
        .finish()
        .unwrap();
    let dtypes = dataframe.dtypes();
    let columns = dataframe.get_column_names_owned();

    output.push_str("<h1>File Prediction</h1>\n\n");
    output.push_str("<form id=\"valform\">\n\n");
    output.push_str("<h2>1. Select your target column</h2>\n\n");
    output.push_str("<h4>Target column needs to be a binary string with YES representing 1 and NO representing 0 (sorry)</h4>\n\n");
    output.push_str("<select name=\"target_col\" id=\"target_col\">");
    for (header, dtype) in columns.iter().zip(dtypes.iter()) {
        output.push_str(&format!("<option value=\"{}\">",&header));
        output.push_str(&header);
        output.push_str(": ");
        output.push_str(&dtype.to_string());
        output.push_str("</option>"); 
    }
    output.push_str("</select>");
    output.push_str("<h2>2. Select your feature columns</h2>\n\n");
    output.push_str("<h4>Only select float or integer (f64, f32, i32, etc) columns, text columns will be dropped automatically</h4>\n\n");
    output.push_str("<select name=\"feature_col\" id=\"feature_col\" multiple>");
    for (header, dtype) in columns.iter().zip(dtypes.iter()) {
        output.push_str(&format!("<option value=\"{}\">",&header));
        output.push_str(&header);
        output.push_str(": ");
        output.push_str(&dtype.to_string());
        output.push_str("</option>"); 
    }
    output.push_str("</select>");
    output.push_str("<h2>3. Select iterations and threshold</h2>\n\n");
    output.push_str("<h4>Iterations are the number of times the algorithm will be run to try to fit the data</h4>\n\n");
    output.push_str("<label for=\"iterations\">Iterations</label>");
    output.push_str("<input name=\"iterations\" value=\"100\" id=\"iterations\" type=\"number\" min=\"1\" step=\"1\">"); 
    output.push_str("</br>");
    output.push_str("<button type=\"submit\">Check Your Prediction!</button>");
    output.push_str("</form>");
    output.push_str("\r\n");
    output
}

#[derive(Deserialize)]
struct Data {
    target_col: Vec<String>,
    feature_col: Vec<String>,
    iterations: Vec<u32>,
}

fn trapezoidal(vals: &Vec<(f32, f32)>) -> f32 {
    /* Trapezoidal function shamelessly copied from linfa metrics
    */
    let mut prev_x = vals[0].0;
    let mut prev_y = vals[0].1;
    let mut integral = 0.0;

    for (x, y) in vals.iter().skip(1) {
        integral += (*x - prev_x) * (prev_y + *y) / 2.0;
        prev_x = *x;
        prev_y = *y;
    }
    integral
}

#[wasm_bindgen]
pub fn process_file(buffer: &[u8], data_json: String) -> String {
    let mut output = String::new(); 
    let data: Data = serde_json::from_str(&data_json).unwrap();
    let cursor = Cursor::new(buffer);
    let dataframe = CsvReader::new(cursor)
        .finish()
        .unwrap();
    let col_target = dataframe.column(&data.target_col[0].to_string()).unwrap();
    let target_high = col_target.str().unwrap().iter().collect::<Vec<_>>();
    let target: ArrayBase<ndarray::OwnedRepr<_>, _> = ArrayBase::from_vec(target_high.to_vec());

    let mut features = dataframe.clone();
    let dtypes = features.dtypes();

    // Lets drop string type columns
    let columns = features.get_column_names_owned();
    for (header, dtype) in columns.iter().zip(dtypes.iter()) {
      if &dtype.to_string() == "str" {
        features = features.drop(&header.to_string()).unwrap();
      }
    }

    // Select our features
    features = features.select(&data.feature_col).unwrap();

    // Cast them to float
    for col_name in features.get_column_names_owned(){
        let casted_col = dataframe.column(&col_name).unwrap()
            .cast(&DataType::Float32)
            .expect("Failed to cast column");
        features.with_column(casted_col).unwrap();        
    }

    // Creating and splitting our dataset in train and validation
    let features_ar = features.to_ndarray::<Float64Type>(IndexOrder::C).unwrap();
    let linfa_dataset = Dataset::new(features_ar,target)
           .map_targets(|x| if *x >= Some("YES") {"YES"} else {"NO"});
    let (train,valid) = linfa_dataset.split_with_ratio(0.4);

    // Let's calculate our ROC curve by iterating the threshold value
    let mut roc: Vec<(f32, f32)> = Vec::new();
    for i in 0..101 {
        let threshold = f64::from(i) * 0.01; 
        let pls = LogisticRegression::default(); 
        let model = pls.max_iterations(data.iterations[0].try_into().unwrap()).gradient_tolerance(0.0001).fit(&train).unwrap(); 
        let predicted = model.set_threshold(threshold).predict(&valid);
        let predicted_vals = predicted.clone().into_raw_vec();
        let ground_truth = valid.targets().clone().into_raw_vec();
        let mut vs_true_positive = 0;
        let mut vs_false_positive = 0;
        let mut vs_true_negative = 0;
        let mut vs_false_negative = 0;

        // The false negative positive matrix could be optimized for sure but I prefer to understand it
        for i in 0..predicted_vals.len(){
            if predicted_vals[i] == ground_truth[i] && predicted_vals[i] == "YES" {
                vs_true_positive += 1;
            }
            else if predicted_vals[i] == ground_truth[i] && predicted_vals[i] == "NO" {
                vs_true_negative += 1;
            }
            else if predicted_vals[i] != ground_truth[i] && predicted_vals[i] == "NO" {
                vs_false_negative += 1;
            }
            else if predicted_vals[i] != ground_truth[i] && predicted_vals[i] == "YES" {
                vs_false_positive += 1;
            }
        }
        let tpr: f32 = (vs_true_positive as f32) / ((vs_true_positive + vs_false_negative) as f32);
        let fpr: f32 = (vs_false_positive as f32) / ((vs_false_positive + vs_true_negative) as f32); 
        roc.push((fpr, tpr));
    }    

    // Is it ok to use the abs value?
    let auc =  trapezoidal(&roc).abs(); 

    let mut buf = String::new();
    {
    let root_area = SVGBackend::with_string(&mut buf, (600, 400)).into_drawing_area();
    root_area.fill(&WHITE).unwrap(); 
    let mut ctx = ChartBuilder::on(&root_area) 
            .set_label_area_size(LabelAreaPosition::Left, 40) 
            .set_label_area_size(LabelAreaPosition::Bottom, 40) 
            .build_cartesian_2d(-0.2..1.2, -0.2..1.2)
            .unwrap();
    ctx.configure_mesh().draw().unwrap();
    ctx.draw_series(
        LineSeries::new(roc.iter().map(|x| (x.0 as f64, x.1 as f64)), &BLACK) 
    ).unwrap(); 
    root_area.present().unwrap();
    }  

    output.push_str(&format!("<h1>Your results are here!</h1>"));
    output.push_str(&format!("<h3>ROC:</h3>"));
    output.push_str(&format!("<img src=\"data:image/svg+xml;base64,{}\">", general_purpose::STANDARD.encode(&buf)));
    output.push_str(&format!("<h3>Area under the curve: {}</h3>", auc));
    output
}
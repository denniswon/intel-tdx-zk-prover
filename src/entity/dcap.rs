use dcap_rs::types::{quotes::body::QuoteBody, TcbStatus, VerifiedOutput};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DcapVerifiedOutput {
    pub quote_version: u16,
    pub tee_type: u32,
    pub tcb_status: TcbStatus,
    pub fmspc: [u8; 6],
    pub quote_body_type: QuoteBodyType,
    pub quote_body_bytes: Vec<u8>,
    pub advisory_ids: Option<Vec<String>>,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub enum QuoteBodyType {
    SGXQuoteBody,
    TD10QuoteBody
}

impl DcapVerifiedOutput {
    pub fn from_output(output: VerifiedOutput) -> DcapVerifiedOutput {
        let quote_body_type = match output.quote_body {
            QuoteBody::SGXQuoteBody(_) => QuoteBodyType::SGXQuoteBody,
            QuoteBody::TD10QuoteBody(_) => QuoteBodyType::TD10QuoteBody
        };
        
        match quote_body_type {
            QuoteBodyType::SGXQuoteBody => {
                if let QuoteBody::SGXQuoteBody(quote_body) = output.quote_body {
                    DcapVerifiedOutput {
                        quote_version: output.quote_version,
                        tee_type: output.tee_type,
                        tcb_status: output.tcb_status,
                        fmspc: output.fmspc,
                        quote_body_type: quote_body_type,
                        quote_body_bytes: quote_body.to_bytes().to_vec(),
                        advisory_ids: output.advisory_ids
                    }
                } else {
                    panic!("Failed to convert quote body to bytes");
                }
            }
            QuoteBodyType::TD10QuoteBody => {
                if let QuoteBody::TD10QuoteBody(quote_body) = output.quote_body {
                    DcapVerifiedOutput {
                        quote_version: output.quote_version,
                        tee_type: output.tee_type,
                        tcb_status: output.tcb_status,
                        fmspc: output.fmspc,
                        quote_body_type: quote_body_type,
                        quote_body_bytes: quote_body.to_bytes().to_vec(),
                        advisory_ids: output.advisory_ids
                    }
                } else {
                    panic!("Failed to convert quote body to bytes");
                }
            }
        }
    }
}
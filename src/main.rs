use idl_parser::EndpointParser as EP;

fn main() {
    let expr = "GET /register/{id:string}/{field:string}?type:string&order:string RQ -> RS";
    print!("{:?}", EP::parse_endpoint(expr))
}

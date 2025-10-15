use gcodekit::gcodeedit::tokenizer::*;

#[tokio::test]
async fn test_tokenizer_basic() {
    let svc = TokenizerService::new(10);
    let _h = svc.start_worker();
    svc.submit_content("G1 X10 Y20 ; move\nM3 S1000");
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let parsed = svc.get_parsed();
    assert_eq!(parsed.len(), 2);
    // First line: G1 X10 Y20 ; move -> tokens: G1,X10,Y20,;move => comment included
    assert!(parsed[0].tokens.iter().any(|t| t.kind == TokenKind::Command));
    assert!(parsed[1].tokens.iter().any(|t| t.kind == TokenKind::Command));
}

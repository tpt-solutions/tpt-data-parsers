use tpt_cron_parse::{CronExpr, CronField, CronFieldName};

#[test]
fn integration_step_on_range() {
    let e = CronExpr::parse("0 1-5/2 * * *").unwrap();
    assert_eq!(
        e.hours,
        CronField::Step(Box::new(CronField::Range(1, 5)), 2)
    );
    assert_eq!(e.hours.expand(0, 23), vec![1, 3, 5]);
}

#[test]
fn integration_6field_step_seconds() {
    let e = CronExpr::parse("*/10 * * * * *").unwrap();
    assert!(e.is_6_field());
    assert_eq!(
        e.seconds,
        Some(CronField::Step(Box::new(CronField::Any), 10))
    );
}

#[test]
fn integration_all_any_fields() {
    let e = CronExpr::parse("* * * * *").unwrap();
    assert_eq!(e.minutes, CronField::Any);
    assert_eq!(e.hours, CronField::Any);
    assert_eq!(e.dom, CronField::Any);
    assert_eq!(e.month, CronField::Any);
    assert_eq!(e.dow, CronField::Any);
}

#[test]
fn integration_dec_31_11pm() {
    let e = CronExpr::parse("0 23 31 12 *").unwrap();
    assert_eq!(e.to_human_readable(), "At 11:00 PM on December 31st");
}

#[test]
fn integration_wednesday_noon() {
    let e = CronExpr::parse("0 12 * * 3").unwrap();
    assert_eq!(e.to_human_readable(), "Every Wednesday at 12:00 PM");
}

#[test]
fn integration_out_of_range_fields_rejected() {
    for expr in [
        "60 * * * *",
        "0 99 * * *",
        "0 0 32 * *",
        "0 0 0 * *",
        "0 0 1 13 *",
        "0 0 1 0 *",
        "0 0 * * 8",
        "60 0 0 * * *",
    ] {
        assert!(CronExpr::parse(expr).is_err(), "{expr} should be rejected");
    }
}

#[test]
fn integration_trailing_garbage_rejected() {
    for expr in [
        "* * * * 1garbage",
        "* * * * *DROP TABLE",
        "0 9 * * 1;rm -rf /",
        "*/5 * * * */",
    ] {
        assert!(CronExpr::parse(expr).is_err(), "{expr} should be rejected");
    }
}

#[test]
fn integration_tab_separated_fields() {
    let e = CronExpr::parse("*/15\t0\t9\t*\t*\t1-5").unwrap();
    assert!(e.is_6_field());
    assert_eq!(e.dow, CronField::Range(1, 5));
    assert_eq!(
        CronExpr::parse("0\t9\t*\t*\t1").unwrap(),
        CronExpr::parse("0 9 * * 1").unwrap()
    );
}

#[test]
fn integration_shorthand_aliases() {
    assert_eq!(
        CronExpr::parse("@weekly").unwrap(),
        CronExpr::parse("0 0 * * 0").unwrap()
    );
    assert_eq!(
        CronExpr::parse("@YEARLY").unwrap().to_human_readable(),
        "At 12:00 AM on January 1st"
    );
    let err = CronExpr::parse("@reboot").unwrap_err();
    assert_eq!(err.field, CronFieldName::Minutes);
    assert!(err.to_string().contains("@reboot"));
}

#[test]
fn integration_named_fields() {
    let named = CronExpr::parse("0 9 * JAN-mar Mon-Fri").unwrap();
    let numeric = CronExpr::parse("0 9 * 1-3 1-5").unwrap();
    assert_eq!(named, numeric);
    assert_eq!(
        CronExpr::parse("0 0 * * SUN").unwrap().dow,
        CronField::Value(0)
    );
    assert!(CronExpr::parse("0 0 * * NOTADAY").is_err());
}

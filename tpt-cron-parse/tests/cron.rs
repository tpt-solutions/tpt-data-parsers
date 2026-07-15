use tpt_cron_parse::{CronExpr, CronField};

#[test]
fn integration_step_on_range() {
    let e = CronExpr::parse("0 1-5/2 * * *").unwrap();
    assert_eq!(
        e.hours,
        CronField::Step(Box::new(CronField::Range(1, 5)), 2)
    );
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

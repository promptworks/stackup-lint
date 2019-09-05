use stackup_lint::{
    self,
    interface::{CheckResult, Comment, Pos, PositionedComment, Severity},
};

#[test]
fn test_check_example() {
    let schema = include_str!("./stackup-example.graphql");
    let check_result = stackup_lint::check(schema);

    let comments = vec![];
    assert_eq!(check_result, CheckResult::new(schema.to_string(), comments));
}

#[test]
fn test_check() {
    let schema = include_str!("./test.graphql");
    let check_result = stackup_lint::check(schema);

    let comments = vec![
        PositionedComment::new(
            Pos {
                line: 12,
                column: 5,
            },
            Comment::new(
                Severity::Error,
                r#"Missing "@belongsTo" directive"#.to_string(),
            ),
        ),
        PositionedComment::new(
            Pos { line: 5, column: 1 },
            Comment::new(
                Severity::Error,
                r#"Missing field "quxs", due to association on object type Qux - 10:1\n"#
                    .to_string(),
            ),
        ),
        PositionedComment::new(
            Pos { line: 1, column: 1 },
            Comment::new(
                Severity::Error,
                "Missing id field on object type Foo, consider adding one".to_string(),
            ),
        ),
        PositionedComment::new(
            Pos {
                line: 15,
                column: 1,
            },
            Comment::new(
                Severity::Warning,
                r#"Consider making this "id: ID!""#.to_string(),
            ),
        ),
        PositionedComment::new(
            Pos { line: 7, column: 5 },
            Comment::new(
                Severity::Warning,
                r#"List of Scalars are not supported You may want an association instead"#
                    .to_string(),
            ),
        ),
    ];
    assert_eq!(check_result, CheckResult::new(schema.to_string(), comments));
}

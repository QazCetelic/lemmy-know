use lemmy_client::lemmy_api_common::lemmy_db_schema::newtypes::{CommentReportId, PostReportId};

pub fn extract_post_report_id(id: &PostReportId) -> i32 {
    let json = serde_json::to_string(&id).expect("Failed to convert to JSON");
    let post_report_id: i32 = json.parse().expect("Failed to convert to JSON");
    post_report_id
}

pub fn extract_comment_report_id(id: &CommentReportId) -> i32 {
    let json = serde_json::to_string(&id).expect("Failed to convert to JSON");
    let comment_report_id: i32 = json.parse().expect("Failed to convert to JSON");
    comment_report_id
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_id() {
        let id = PostReportId::default();
        let int = extract_post_report_id(&id);
        assert_eq!(0, int);
    }

    #[test]
    fn comment_id() {
        let id = CommentReportId::default();
        let int = extract_comment_report_id(&id);
        assert_eq!(0, int);
    }
}
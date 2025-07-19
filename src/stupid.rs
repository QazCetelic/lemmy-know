use lemmy_client::lemmy_api_common::lemmy_db_schema::newtypes::{CommentReportId, PostReportId};

// The integer value of the newtype is not exposed
// Diesel can work with newtypes, but that feature seems to be unavailable through the crate
// I can't guarantee the memory layout (requires #[repr(transparent)]) so transmutate doesn't work
// But this works and I'm not interested in finding a better method after wasting 30 minutes already

pub fn extract_post_report_id(id: PostReportId) -> i32 {
    let json = serde_json::to_string(&id).expect("Failed to convert to JSON");
    let post_report_id: i32 = json.parse().expect("Failed to convert to JSON");
    post_report_id
}

pub fn extract_comment_report_id(id: CommentReportId) -> i32 {
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
        let int = extract_post_report_id(id);
        assert_eq!(0, int);
    }

    #[test]
    fn comment_id() {
        let id = CommentReportId::default();
        let int = extract_comment_report_id(id);
        assert_eq!(0, int);
    }
}
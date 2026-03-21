/// A single curl-quest challenge.
pub struct Quest {
    pub id: usize,
    pub title: &'static str,
    pub instructions: &'static str,
    pub hint: &'static str,
    pub verify: fn(&str) -> VerifyResult,
}

pub enum VerifyResult {
    Pass,
    Fail(String),
}

/// Look up a quest by 1-based id.
pub fn get(id: usize) -> Option<&'static Quest> {
    QUESTS.iter().find(|q| q.id == id)
}

#[allow(dead_code)]
pub fn all() -> &'static [Quest] {
    &QUESTS
}

// ---------------------------------------------------------------------------
// Verification helpers
// ---------------------------------------------------------------------------

fn check(output: &str, needle: &str, msg: &str) -> VerifyResult {
    if output.contains(needle) {
        VerifyResult::Pass
    } else {
        VerifyResult::Fail(msg.to_string())
    }
}

// ---------------------------------------------------------------------------
// Per-quest verify functions
// ---------------------------------------------------------------------------

fn v01(o: &str) -> VerifyResult {
    check(o, "\"url\"", "Expected a JSON response with a 'url' field. Try: curl http://httpbin.org/get")
}
fn v02(o: &str) -> VerifyResult {
    check(o, "HTTP/", "Expected HTTP headers in the output. Use the -I flag.")
}
fn v03(o: &str) -> VerifyResult {
    check(o, "* Connected", "Expected verbose connection info. Use the -v flag.")
}
fn v04(o: &str) -> VerifyResult {
    check(o, "\"Content-Type\": \"application/json\"", "The response should echo back 'Content-Type: application/json'. Set it with -H.")
}
fn v05(o: &str) -> VerifyResult {
    check(o, "\"name\": \"curl\"", "Expected form field name=curl in the response. Use -d \"name=curl\".")
}
fn v06(o: &str) -> VerifyResult {
    check(o, "\"url\"", "Curl should have followed the redirect and returned a JSON body. Add -L.")
}
fn v07(o: &str) -> VerifyResult {
    check(o, "\"user-agent\"", "Expected the user-agent field in the JSON response.")
}
fn v08(o: &str) -> VerifyResult {
    check(o, "CurlQuests/1.0", "The User-Agent must be exactly 'CurlQuests/1.0'. Use -A \"CurlQuests/1.0\".")
}
fn v09(o: &str) -> VerifyResult {
    check(o, "\"authenticated\": true", "Authentication failed. Use -u user:pass.")
}
fn v10(o: &str) -> VerifyResult {
    check(o, "\"authenticated\": true", "Bearer auth failed. Use -H \"Authorization: Bearer any-token\".")
}
fn v11(o: &str) -> VerifyResult {
    if o.contains("\"quest\": \"11\"") || o.contains("\"quest\":\"11\"") {
        VerifyResult::Pass
    } else {
        VerifyResult::Fail("Expected query param quest=11 echoed back. Try: curl \"http://httpbin.org/get?quest=11\"".into())
    }
}
fn v12(o: &str) -> VerifyResult {
    check(o, "\"X-Custom-Header\": \"curl-quests\"", "Header not found. Add -H \"X-Custom-Header: curl-quests\".")
}
fn v13(o: &str) -> VerifyResult {
    check(o, "\"json\":", "Expected JSON body echoed in PUT response.")
}
fn v14(o: &str) -> VerifyResult {
    check(o, "\"url\":", "Expected a successful DELETE response body.")
}
fn v15(o: &str) -> VerifyResult {
    check(o, "\"json\":", "Expected JSON body echoed in PATCH response.")
}
fn v16(o: &str) -> VerifyResult {
    check(o, "\"cookies\":", "Expected cookies in the response. Use -c and -b flags with -L.")
}
fn v17(o: &str) -> VerifyResult {
    if o.contains("\"url\"") || o.contains("200") {
        VerifyResult::Pass
    } else {
        VerifyResult::Fail("Expected a successful response. Add --max-time 5.".into())
    }
}
fn v18(o: &str) -> VerifyResult {
    check(o, "\"gzipped\": true", "Expected gzipped: true in the response. Use --compressed.")
}
fn v19(o: &str) -> VerifyResult {
    check(o, "\"url\"", "Expected a clean JSON response. Use -s to silence progress output.")
}
fn v20(o: &str) -> VerifyResult {
    let t = o.trim();
    if t == "200" || t.ends_with("200") {
        VerifyResult::Pass
    } else {
        VerifyResult::Fail(format!("Expected only '200', got: '{}'. Try: curl -s -o /dev/null -w \"%{{http_code}}\" URL", t))
    }
}
fn v21(o: &str) -> VerifyResult {
    if o.matches("\"uuid\"").count() >= 2 {
        VerifyResult::Pass
    } else {
        VerifyResult::Fail("Expected two UUID responses. Pass two URLs to a single curl command.".into())
    }
}
fn v22(o: &str) -> VerifyResult {
    check(o, "\"files\":", "Expected file upload response. Use -F \"file=@/path/to/file\".")
}
fn v23(o: &str) -> VerifyResult {
    check(o, "\"url\"", "Expected a successful response. Use --retry 3.")
}
fn v24(o: &str) -> VerifyResult {
    if o.contains("\"url\"") && (o.contains("\"headers\"") || o.contains("* Connected")) {
        VerifyResult::Pass
    } else {
        VerifyResult::Fail("Need both a JSON body with 'url' and either headers or verbose output.".into())
    }
}

// ---------------------------------------------------------------------------
// Quest catalog
// ---------------------------------------------------------------------------

static QUESTS: [Quest; 24] = [
    Quest {
        id: 1,
        title: "Your First Request",
        instructions: "Make a basic GET request to:\n\n  http://httpbin.org/get\n\nThis is the most fundamental curl command. You should see a JSON response containing details about your request such as headers, IP, and URL.",
        hint: "curl http://httpbin.org/get",
        verify: v01,
    },
    Quest {
        id: 2,
        title: "Fetch Only Headers",
        instructions: "Sometimes you only want the HTTP response headers, not the body.\n\nFetch only the headers from:\n\n  http://httpbin.org/get\n\nThe output should start with something like 'HTTP/2 200'.",
        hint: "Use the -I flag (uppercase i)",
        verify: v02,
    },
    Quest {
        id: 3,
        title: "Verbose Mode",
        instructions: "See everything curl does under the hood — connection details, TLS handshake, request and response headers.\n\nMake a verbose request to:\n\n  http://httpbin.org/get\n\nLook for lines starting with * (connection info), > (request), < (response).",
        hint: "Use the -v flag",
        verify: v03,
    },
    Quest {
        id: 4,
        title: "POST with JSON",
        instructions: "Send a POST request with a JSON body to:\n\n  http://httpbin.org/post\n\nSet the Content-Type header to 'application/json' and include any JSON payload. The server will echo your Content-Type back in the response.",
        hint: "curl -X POST -H \"Content-Type: application/json\" -d '{\"key\":\"value\"}' http://httpbin.org/post",
        verify: v04,
    },
    Quest {
        id: 5,
        title: "POST Form Data",
        instructions: "Send a POST request with URL-encoded form data to:\n\n  http://httpbin.org/post\n\nInclude a field named 'name' with the value 'curl'. The server will echo your form fields back in the 'form' key of the response.",
        hint: "curl -X POST -d \"name=curl\" http://httpbin.org/post",
        verify: v05,
    },
    Quest {
        id: 6,
        title: "Follow Redirects",
        instructions: "By default curl does NOT follow HTTP redirects (3xx responses).\n\nFetch this URL and make curl follow the redirect automatically:\n\n  http://httpbin.org/redirect/1\n\nYou should get the final JSON response, not a redirect message.",
        hint: "Add the -L flag to follow redirects",
        verify: v06,
    },
    Quest {
        id: 7,
        title: "Check Your User-Agent",
        instructions: "Every HTTP client sends a User-Agent header that identifies itself.\n\nMake a request to:\n\n  http://httpbin.org/user-agent\n\nThis endpoint returns the User-Agent header that curl sent. What does curl identify itself as by default?",
        hint: "curl http://httpbin.org/user-agent",
        verify: v07,
    },
    Quest {
        id: 8,
        title: "Custom User-Agent",
        instructions: "Set a custom User-Agent to identify your client.\n\nMake a request to:\n\n  http://httpbin.org/user-agent\n\nWith the User-Agent set to exactly: CurlQuests/1.0",
        hint: "Use -A \"CurlQuests/1.0\" or -H \"User-Agent: CurlQuests/1.0\"",
        verify: v08,
    },
    Quest {
        id: 9,
        title: "Basic Authentication",
        instructions: "Many APIs use HTTP Basic Authentication.\n\nMake an authenticated request to:\n\n  http://httpbin.org/basic-auth/user/pass\n\nUsing username 'user' and password 'pass'. The response should contain: \"authenticated\": true",
        hint: "Use -u user:pass",
        verify: v09,
    },
    Quest {
        id: 10,
        title: "Bearer Token Auth",
        instructions: "Modern APIs often use Bearer tokens in the Authorization header.\n\nMake a request to:\n\n  http://httpbin.org/bearer\n\nWith any Bearer token value in the Authorization header.",
        hint: "curl -H \"Authorization: Bearer mytoken\" http://httpbin.org/bearer",
        verify: v10,
    },
    Quest {
        id: 11,
        title: "Query Parameters",
        instructions: "Send a request with query parameters in the URL.\n\nMake a GET request to httpbin's /get endpoint with the query parameter:\n\n  quest=11\n\nThe server will echo the args back in the response.",
        hint: "curl \"http://httpbin.org/get?quest=11\"  (quote the URL!)",
        verify: v11,
    },
    Quest {
        id: 12,
        title: "Custom Request Header",
        instructions: "Add a custom header to your request.\n\nMake a request to:\n\n  http://httpbin.org/headers\n\nWith the header:  X-Custom-Header: curl-quests\n\nThe server echoes all request headers back.",
        hint: "Use -H \"X-Custom-Header: curl-quests\"",
        verify: v12,
    },
    Quest {
        id: 13,
        title: "PUT Request",
        instructions: "Send a PUT request — typically used to replace an entire resource.\n\nMake a PUT request with a JSON body to:\n\n  http://httpbin.org/put\n\nInclude a Content-Type: application/json header and any JSON payload.",
        hint: "curl -X PUT -H \"Content-Type: application/json\" -d '{\"key\":\"value\"}' http://httpbin.org/put",
        verify: v13,
    },
    Quest {
        id: 14,
        title: "DELETE Request",
        instructions: "Send a DELETE request — used to remove a resource.\n\nMake a DELETE request to:\n\n  http://httpbin.org/delete\n\nThe response will be a JSON object confirming the request.",
        hint: "curl -X DELETE http://httpbin.org/delete",
        verify: v14,
    },
    Quest {
        id: 15,
        title: "PATCH Request",
        instructions: "Send a PATCH request — used for partial updates to a resource.\n\nMake a PATCH request with a JSON body to:\n\n  http://httpbin.org/patch",
        hint: "curl -X PATCH -H \"Content-Type: application/json\" -d '{\"key\":\"new\"}' http://httpbin.org/patch",
        verify: v15,
    },
    Quest {
        id: 16,
        title: "Cookies",
        instructions: "Save cookies from a response and send them back.\n\nVisit the cookie-setting URL (with -L to follow the redirect):\n\n  http://httpbin.org/cookies/set/session/abc\n\nUse -c to save cookies and -b to send them.",
        hint: "curl -L -c /tmp/cq_cookies.txt -b /tmp/cq_cookies.txt http://httpbin.org/cookies/set/session/abc",
        verify: v16,
    },
    Quest {
        id: 17,
        title: "Request Timeout",
        instructions: "Prevent curl from waiting forever by setting a timeout.\n\nMake a request to:\n\n  http://httpbin.org/delay/1\n\nWith a maximum time of 5 seconds. The endpoint delays the response by 1 second intentionally.",
        hint: "Add --max-time 5 to your curl command",
        verify: v17,
    },
    Quest {
        id: 18,
        title: "Compressed Responses",
        instructions: "Servers can compress responses to save bandwidth. Tell curl to request and automatically decompress gzip responses.\n\nMake a request to:\n\n  http://httpbin.org/gzip\n\nThe response should contain: \"gzipped\": true",
        hint: "Add --compressed to your curl command",
        verify: v18,
    },
    Quest {
        id: 19,
        title: "Silent Mode",
        instructions: "By default curl shows a progress bar on stderr. Suppress it with silent mode.\n\nMake a silent request to:\n\n  http://httpbin.org/get\n\nOnly the response body should appear in the output.",
        hint: "Use the -s flag",
        verify: v19,
    },
    Quest {
        id: 20,
        title: "Extract the Status Code",
        instructions: "Sometimes you only need the HTTP status code, not the body.\n\nMake a request to:\n\n  http://httpbin.org/status/200\n\nOutput ONLY the numeric status code. Your output should be exactly: 200",
        hint: "curl -s -o /dev/null -w \"%{http_code}\" http://httpbin.org/status/200",
        verify: v20,
    },
    Quest {
        id: 21,
        title: "Multiple URLs",
        instructions: "curl can fetch multiple URLs in a single command.\n\nFetch both of these URLs with one curl command:\n\n  http://httpbin.org/uuid\n  http://httpbin.org/uuid\n\nYou should see two different UUID values in the output.",
        hint: "Just list both URLs after curl, separated by a space",
        verify: v21,
    },
    Quest {
        id: 22,
        title: "File Upload",
        instructions: "Upload a file using multipart form data (the same way HTML forms upload files).\n\nUpload any file to:\n\n  http://httpbin.org/post\n\nThe response should contain a 'files' key with your uploaded content.",
        hint: "curl -F \"file=@/etc/hostname\" http://httpbin.org/post",
        verify: v22,
    },
    Quest {
        id: 23,
        title: "Automatic Retry",
        instructions: "Network requests sometimes fail transiently. Configure curl to retry automatically.\n\nMake a request to:\n\n  http://httpbin.org/get\n\nWith automatic retry up to 3 times on failure.",
        hint: "Add --retry 3 to your command",
        verify: v23,
    },
    Quest {
        id: 24,
        title: "The Final Challenge",
        instructions: "Combine what you've learned.\n\nMake a request to:\n\n  http://httpbin.org/anything\n\nRequirements:\n• Use verbose mode (-v)\n• Add at least one custom header\n• The response must contain both 'url' and 'headers'\n\nThis endpoint echoes everything back.",
        hint: "curl -v -H \"X-Quest: complete\" http://httpbin.org/anything",
        verify: v24,
    },
];

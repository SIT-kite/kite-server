
fn main() {
    let out_dir = std::path::PathBuf::from("src/service/gen/");

    tonic_build::configure()
        .build_server(true)
        .out_dir(out_dir)
        .compile(&[
            // Basic
                "proto/template.proto",
                "proto/token.proto",
                "proto/typing.proto",
            // Test
                "proto/ping.proto",
            // Service
                "proto/badge.proto",
                "proto/balance.proto",
                "proto/board.proto",
                "proto/classroom_browser.proto",
                "proto/exception.proto",
                "proto/freshman.proto",
                "proto/game.proto",
                "proto/user.proto",
                "proto/yellow_page.proto",
        ],
                 &["proto"])
        .unwrap();
}

/*
 * 上应小风筝  便利校园，一步到位
 * Copyright (C) 2020-2023 上海应用技术大学 上应小风筝团队
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

fn main() {
    let out_dir = std::path::PathBuf::from("src/service/gen/");
    println!("Run build.rs !!!");

    let proto_files = &[
        // Basic
        "../proto/template.proto",
        "../proto/token.proto",
        "../proto/typing.proto",
        // Test
        "../proto/ping.proto",
        // Service
        "../proto/badge.proto",
        "../proto/balance.proto",
        "../proto/board.proto",
        "../proto/captcha.proto",
        "../proto/classroom_browser.proto",
        "../proto/exception.proto",
        "../proto/freshman.proto",
        "../proto/game.proto",
        "../proto/user.proto",
        "../proto/yellow_page.proto",
    ];
    for &path in proto_files {
        println!("cargo:rerun-if-changed=\"{path}\"");
    }

    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .file_descriptor_set_path("../target/compiled-descriptor.bin")
        .out_dir(out_dir)
        .compile(proto_files, &["../proto"])
        .unwrap();
}

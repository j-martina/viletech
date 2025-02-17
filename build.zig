const std = @import("std");
const builtin = @import("builtin");

pub const zon = @embedFile("build.zig.zon");

pub const ccdb = @import("depend/ccdb.zig");
pub const cimgui = @import("depend/build.cimgui.zig");
pub const datetime = @import("depend/datetime.zig");
pub const Sdl = @import("sdl");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const assets = b.createModule(.{
        .root_source_file = b.path("assets/assets.zig"),
        .target = target,
        .optimize = optimize,
    });
    const deque = b.createModule(.{
        .root_source_file = b.path("depend/deque.zig"),
        .target = target,
        .optimize = optimize,
    });
    const zig_args = b.dependency("zig-args", .{});

    const check = b.step("check", "Semantic check for ZLS");

    const testx = b.option(
        []const []const u8,
        "testx",
        "Extra features to test",
    ) orelse &[0][]const u8{};

    const test_filters = b.option(
        []const []const u8,
        "test-filters",
        "Which subset of unit tests should be run",
    ) orelse &[0][]const u8{};

    const test_step = b.step("test", "Run unit test suite");
    engine.tests(b, target, optimize, test_step, test_filters);
    doomparse.tests(b, target, optimize, test_step, test_filters);
    subterra.tests(b, target, optimize, test_step, test_filters, testx);
    vilefs.tests(b, target, optimize, test_step, test_filters);
    wadload.tests(b, target, optimize, test_step, test_filters);
    zbcx.tests(b, target, optimize, test_step, test_filters);
    zdfs.tests(b, target, optimize, test_step, test_filters);
    zmsx.tests(b, target, optimize, test_step, test_filters);

    const doc_step = b.step("doc", "Generate documentation");
    engine.doc(b, target, optimize, doc_step);
    doomparse.doc(b, target, optimize, doc_step);
    subterra.doc(b, target, optimize, doc_step);
    wadload.doc(b, target, optimize, doc_step);
    zbcx.doc(b, target, optimize, doc_step);
    zdfs.doc(b, target, optimize, doc_step);

    const re2_step = b.step("re2", "Run all re2zig lexer generators");
    subterra.generateUdmfLexer(b, re2_step);

    var client_builder = @import("client/Builder.zig"){
        .b = b,
        .target = target,
        .optimize = optimize,
        .check = check,

        .libdumb = b.option(bool, "dumb", "Use libDUMB if available") orelse
            true,
        .libfluidsynth = b.option(bool, "fluidsynth", "Use libfluidsynth if available") orelse
            true,
        .libsdlimage = b.option(bool, "image", "Use libsdlimage if available") orelse
            true,
        .libmad = b.option(bool, "mad", "Use libmad if available") orelse
            true,
        .libportmidi = b.option(bool, "portmidi", "Use libportmidi if available") orelse
            true,
        .libvorbisfile = b.option(bool, "vorbisfile", "Use libvorbisfile if available") orelse
            true,

        .assets = assets,
        .deque = deque,
        .sdl = Sdl.init(b, null, null),
        .zig_args = zig_args.module("args"),
    };
    const client = client_builder.build();

    const demotest_step = b.step("demotest", "Run demo accuracy regression tests");

    const demotest = b.addTest(.{
        .root_source_file = b.path("demotest/main.zig"),
        .target = target,
        // Optimization level of the client gets decided by what user passes to
        // `-D`. Don't optimize the unit test binary, since it just takes more
        // time and has no benefit.
        .optimize = .Debug,
    });
    demotest.step.dependOn(&client.step);

    const demotest_in = b.addOptions();
    demotest_in.addOption([]const u8, "install_prefix", b.install_prefix);
    demotest.root_module.addOptions("cfg", demotest_in);

    const run_demotest = b.addRunArtifact(demotest);
    run_demotest.has_side_effects = true;
    demotest_step.dependOn(&run_demotest.step);
}

pub const engine = struct {
    pub fn link(b: *std.Build, compile: *std.Build.Step.Compile, name: ?[]const u8) void {
        compile.root_module.addImport(
            name orelse "viletech",
            b.addModule("viletech", .{
                .root_source_file = b.path("engine/src/root.zig"),
            }),
        );
    }

    fn doc(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        doc_step: *std.Build.Step,
    ) void {
        const dummy = b.addStaticLibrary(.{
            .name = "viletech",
            .root_source_file = b.path("engine/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        const install_docs = b.addInstallDirectory(.{
            .source_dir = dummy.getEmittedDocs(),
            .install_dir = .{ .custom = "docs" },
            .install_subdir = "viletech",
        });

        doc_step.dependOn(&install_docs.step);
    }

    fn tests(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        test_step: *std.Build.Step,
        filters: []const []const u8,
    ) void {
        const unit_tests = b.addTest(.{
            .root_source_file = b.path("engine/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        unit_tests.filters = filters;

        const run_unit_tests = b.addRunArtifact(unit_tests);
        test_step.dependOn(&run_unit_tests.step);
    }
};

// Libraries ///////////////////////////////////////////////////////////////////

pub const doomparse = struct {
    pub fn link(b: *std.Build, compile: *std.Build.Step.Compile, name: ?[]const u8) void {
        const module = b.addModule("doomparse", .{
            .root_source_file = b.path("libs/doomparse/src/root.zig"),
        });

        module.addImport("deque", b.addModule("deque", .{
            .root_source_file = b.path("depend/deque.zig"),
        }));

        compile.root_module.addImport(name orelse "doomparse", module);
    }

    fn doc(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        doc_step: *std.Build.Step,
    ) void {
        const dummy = b.addStaticLibrary(.{
            .name = "doomparse",
            .root_source_file = b.path("libs/doomparse/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        const install_docs = b.addInstallDirectory(.{
            .source_dir = dummy.getEmittedDocs(),
            .install_dir = .{ .custom = "docs" },
            .install_subdir = "doomparse",
        });

        doc_step.dependOn(&install_docs.step);
    }

    fn tests(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        test_step: *std.Build.Step,
        filters: []const []const u8,
    ) void {
        const unit_tests = b.addTest(.{
            .root_source_file = b.path("libs/doomparse/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        unit_tests.root_module.addImport("deque", b.addModule("deque", .{
            .root_source_file = b.path("depend/deque.zig"),
        }));

        unit_tests.filters = filters;

        const run_unit_tests = b.addRunArtifact(unit_tests);
        test_step.dependOn(&run_unit_tests.step);
    }
};

pub const subterra = struct {
    pub fn link(b: *std.Build, compile: *std.Build.Step.Compile, config: struct {
        name: []const u8 = "subterra",
        znbx: bool,
    }) void {
        const module = b.addModule("subterra", .{
            .root_source_file = b.path("libs/subterra/src/root.zig"),
        });

        const opts = b.addOptions();
        opts.addOption(bool, "znbx", config.znbx);
        compile.root_module.addOptions("cfg", opts);

        compile.root_module.addImport(config.name, module);

        if (config.znbx) znbx.link(b, compile, .{
            .name = "znbx",
            .target = compile.root_module.resolved_target.?,
            .optimize = compile.root_module.optimize.?,
        });
    }

    fn doc(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        doc_step: *std.Build.Step,
    ) void {
        const dummy = b.addStaticLibrary(.{
            .name = "subterra",
            .root_source_file = b.path("libs/subterra/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        const install_docs = b.addInstallDirectory(.{
            .source_dir = dummy.getEmittedDocs(),
            .install_dir = .{ .custom = "docs" },
            .install_subdir = "subterra",
        });

        doc_step.dependOn(&install_docs.step);
    }

    fn tests(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        test_step: *std.Build.Step,
        filters: []const []const u8,
        testx: []const []const u8,
    ) void {
        const unit_tests = b.addTest(.{
            .root_source_file = b.path("libs/subterra/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        unit_tests.filters = filters;

        var dmxgus: []const u8 = "";
        var genmidi: []const u8 = "";
        var test_znbx = false;

        for (testx) |s| {
            if (std.mem.eql(u8, std.fs.path.stem(s), "GENMIDI")) {
                genmidi = s;
            } else if (std.mem.eql(u8, std.fs.path.stem(s), "DMXGUS")) {
                dmxgus = s;
            } else if (std.mem.eql(u8, s, "znbx")) {
                test_znbx = true;
            }
        }

        const opts = b.addOptions();
        opts.addOption([]const u8, "dmxgus_sample", dmxgus);
        opts.addOption([]const u8, "genmidi_sample", genmidi);
        opts.addOption(bool, "znbx", test_znbx);
        unit_tests.root_module.addOptions("cfg", opts);

        if (test_znbx) {
            znbx.link(b, unit_tests, .{
                .name = "znbx",
                .target = target,
                .optimize = optimize,
            });
        }

        const run_unit_tests = b.addRunArtifact(unit_tests);
        test_step.dependOn(&run_unit_tests.step);
    }

    fn generateUdmfLexer(b: *std.Build, re2_step: *std.Build.Step) void {
        const run = b.addSystemCommand(&[_][]const u8{
            "re2zig",
            "--lang",
            "zig",
            "--api",
            "default",
            "-i",
            "--loop-switch",
            "--case-ranges",
            "-W",
            "libs/subterra/src/UdmfLexer.zig.re",
            "-o",
            "libs/subterra/src/UdmfLexer.zig",
        });

        run.addFileInput(b.path("libs/subterra/src/UdmfLexer.zig.re"));
        re2_step.dependOn(&run.step);
    }
};

pub const vilefs = struct {
    pub fn link(b: *std.Build, compile: *std.Build.Step.Compile, name: ?[]const u8) void {
        const module = b.addModule("vilefs", .{
            .root_source_file = b.path("libs/vilefs/src/root.zig"),
        });

        module.addImport("wadload", wadload.module(b, null));

        compile.root_module.addImport(name orelse "vilefs", module);
    }

    fn tests(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        test_step: *std.Build.Step,
        filters: []const []const u8,
    ) void {
        const unit_tests = b.addTest(.{
            .root_source_file = b.path("libs/vilefs/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        unit_tests.root_module.addImport("wadload", wadload.module(b, null));

        unit_tests.filters = filters;

        const run_unit_tests = b.addRunArtifact(unit_tests);
        test_step.dependOn(&run_unit_tests.step);
    }

    fn doc(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        doc_step: *std.Build.Step,
    ) void {
        const dummy = b.addStaticLibrary(.{
            .name = "vilefs",
            .root_source_file = b.path("libs/vilefs/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        const install_docs = b.addInstallDirectory(.{
            .source_dir = dummy.getEmittedDocs(),
            .install_dir = .{ .custom = "docs" },
            .install_subdir = "vilefs",
        });

        doc_step.dependOn(&install_docs.step);
    }
};

pub const wadload = struct {
    pub fn link(b: *std.Build, compile: *std.Build.Step.Compile, name: ?[]const u8) void {
        compile.root_module.addImport(
            name orelse "wadload",
            b.addModule("wadload", .{
                .root_source_file = b.path("libs/wadload/src/root.zig"),
            }),
        );
    }

    pub fn module(b: *std.Build, name: ?[]const u8) *std.Build.Module {
        return b.addModule(name orelse "wadload", .{
            .root_source_file = b.path("libs/wadload/src/root.zig"),
        });
    }

    fn doc(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        doc_step: *std.Build.Step,
    ) void {
        const dummy = b.addStaticLibrary(.{
            .name = "wadload",
            .root_source_file = b.path("libs/wadload/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        const install_docs = b.addInstallDirectory(.{
            .source_dir = dummy.getEmittedDocs(),
            .install_dir = .{ .custom = "docs" },
            .install_subdir = "wadload",
        });

        doc_step.dependOn(&install_docs.step);
    }

    fn tests(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        test_step: *std.Build.Step,
        filters: []const []const u8,
    ) void {
        const unit_tests = b.addTest(.{
            .root_source_file = b.path("libs/wadload/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        unit_tests.filters = filters;

        const run_unit_tests = b.addRunArtifact(unit_tests);
        test_step.dependOn(&run_unit_tests.step);
    }
};

pub const zbcx = struct {
    const c = @import("depend/build.zbcx.zig");

    pub fn link(b: *std.Build, compile: *std.Build.Step.Compile, config: struct {
        name: []const u8 = "zbcx",
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
    }) void {
        c.link(b, compile, config.target, config.optimize);

        const module = b.addModule("zbcx", .{
            .root_source_file = b.path("libs/zbcx/src/root.zig"),
        });

        compile.root_module.addImport(config.name, module);
    }

    fn doc(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        doc_step: *std.Build.Step,
    ) void {
        const dummy = b.addStaticLibrary(.{
            .name = "zbcx",
            .root_source_file = b.path("libs/zbcx/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        c.link(b, dummy, target, optimize);

        const install_docs = b.addInstallDirectory(.{
            .source_dir = dummy.getEmittedDocs(),
            .install_dir = .{ .custom = "docs" },
            .install_subdir = "zbcx",
        });

        doc_step.dependOn(&install_docs.step);
    }

    fn tests(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        test_step: *std.Build.Step,
        filters: []const []const u8,
    ) void {
        const dep = b.dependency("zbcx", .{});

        const unit_tests = b.addTest(.{
            .root_source_file = b.path("libs/zbcx/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        unit_tests.root_module.addAnonymousImport(
            "zbcx/test/stack.bcs",
            .{ .root_source_file = dep.path("test/stack.bcs") },
        );
        unit_tests.root_module.addAnonymousImport(
            "zbcx/lib/zcommon.bcs",
            .{ .root_source_file = dep.path("lib/zcommon.bcs") },
        );
        unit_tests.root_module.addAnonymousImport(
            "zbcx/lib/zcommon.h.bcs",
            .{ .root_source_file = dep.path("lib/zcommon.h.bcs") },
        );

        c.link(b, unit_tests, target, optimize);

        unit_tests.filters = filters;

        const run_unit_tests = b.addRunArtifact(unit_tests);
        test_step.dependOn(&run_unit_tests.step);
    }
};

pub const zdfs = struct {
    const c = @import("depend/build.zdfs.zig");

    pub fn link(b: *std.Build, compile: *std.Build.Step.Compile, config: struct {
        name: []const u8 = "zdfs",
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
    }) void {
        c.link(b, compile, config.target, config.optimize);

        const module = b.addModule("zdfs", .{
            .root_source_file = b.path("libs/zdfs/src/root.zig"),
        });

        compile.root_module.addImport(config.name, module);
    }

    fn doc(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        doc_step: *std.Build.Step,
    ) void {
        const dummy = b.addStaticLibrary(.{
            .name = "zdfs",
            .root_source_file = b.path("libs/zdfs/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        c.link(b, dummy, target, optimize);

        const install_docs = b.addInstallDirectory(.{
            .source_dir = dummy.getEmittedDocs(),
            .install_dir = .{ .custom = "docs" },
            .install_subdir = "zdfs",
        });

        doc_step.dependOn(&install_docs.step);
    }

    fn tests(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        test_step: *std.Build.Step,
        filters: []const []const u8,
    ) void {
        const unit_tests = b.addTest(.{
            .root_source_file = b.path("libs/zdfs/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        c.link(b, unit_tests, target, optimize);

        unit_tests.filters = filters;

        const run_unit_tests = b.addRunArtifact(unit_tests);
        test_step.dependOn(&run_unit_tests.step);
    }
};

pub const zmsx = struct {
    const c = @import("depend/build.zmsx.zig");

    pub fn link(b: *std.Build, compile: *std.Build.Step.Compile, config: struct {
        name: []const u8 = "zmsx",
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
    }) void {
        c.link(b, compile, config.target, config.optimize);

        const module = b.addModule("zmsx", .{
            .root_source_file = b.path("libs/zmsx/src/root.zig"),
        });

        compile.root_module.addImport(config.name, module);
    }

    fn tests(
        b: *std.Build,
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
        test_step: *std.Build.Step,
        filters: []const []const u8,
    ) void {
        const unit_tests = b.addTest(.{
            .root_source_file = b.path("libs/zmsx/src/root.zig"),
            .target = target,
            .optimize = optimize,
        });

        c.link(b, unit_tests, target, optimize);

        unit_tests.filters = filters;

        const run_unit_tests = b.addRunArtifact(unit_tests);
        test_step.dependOn(&run_unit_tests.step);
    }
};

pub const znbx = struct {
    const c = @import("depend/build.znbx.zig");

    pub fn link(b: *std.Build, compile: *std.Build.Step.Compile, config: struct {
        name: []const u8 = "znbx",
        target: std.Build.ResolvedTarget,
        optimize: std.builtin.OptimizeMode,
    }) void {
        c.link(b, compile, config.target, config.optimize);
    }
};

pub fn packageVersion() []const u8 {
    const zon_vers_start = std.mem.indexOf(u8, zon, ".version = ").?;
    const zon_vers_end = std.mem.indexOfPos(u8, zon, zon_vers_start, ",").?;
    const zon_vers_kvp = zon[zon_vers_start..zon_vers_end];
    return std.mem.trim(u8, zon_vers_kvp, ".version =\"");
}

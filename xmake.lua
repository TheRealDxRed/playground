set_project("playground")
set_version("0.0.0")
add_rules("mode.debug", "mode.release")

if is_mode("debug") then
	set_optimize("none")
elseif is_mode("release") then
end

add_includedirs("targets")
includes("targets/*")
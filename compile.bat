@echo off

call compress_ressources.bat

rd /s /q java\src\main\java\army
md java\src\main\java\army\warfare\skiter
copy /b src\java_glue.rs.in +,,

cargo build --release
REM cargo build --release --target x86_64-pc-windows-gnu

REM copy /Y target\release\libskiter.so java\src\main\resources\
copy /Y target\release\skiter.dll java\src\main\resources\

cd java
call gradlew build
cd..

copy /Y java\build\libs\skiter-1.0-SNAPSHOT.jar .\
# SAVE-ME-FILES
Console line application for making backups.

Application copies files from one place to another while preserving
relative directory structure.<br>
It allows to filter files by their name suffixes.<br>
It allows to exclude directories from copying.

```
../
|- SRC_DIRECTORY
   |- root_file.txt
   |- child
      |- child_file.txt
   |- ignored_directory
      |- ignored_file.txt
   
|- DST_DIRECTORY
   |- root_file.txt
   |- child
      |- child_file.txt
```

### Suffixes
Each suffix need to be in separate line.<br>
Empty lines and lines starting with '//' are ignored.
```
//-----------
// text files
//-----------
.txt
.csv

//-------
// images
//-------
.jpg
.png

//------------------
// anything you want
//------------------
_screenshot
exact_filename.backup.tex
```

### Exclusions
Each exclusion need to be in separate line.<br>
Empty lines and lines starting with '//' are ignored.<br>
Exclusion need to be an absolute path to existing directory
```
//-------------
// system paths
//-------------
/tmp
/dev
/etc

//----------------------------
// any path you want to ignore
//----------------------------
/home/name/.cache
/home/name/.cargo
```

## Usage
copy all files
> save-me-files -s SRC -d DST

copy all files excluding those at specified paths
> save-me-files -s SRC -d DST -e EXCLUDE_PATHS_FILE

copy files with specified suffixes
> save-me-files -s SRC -d DST -i INCLUDE_SUFFIXES_FILE

Before copying files application logs all paths that will be copied.
It is possible to stop before copying starts. It's useful when you need
to know precisely what files will be copied beforehand.
> save-me-files -s SRC -d DST --no-copy

Application logs everything to standard output so it's possible to redirect output to the file like so
> save-me-files -s SRC -d DST --no-copy > output.log
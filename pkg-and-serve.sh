A=happs-trailz.tar
B=trailz_ui-pkg.tar
tar cvf $A ./happs/trailz/*
tar cvf $B ./crates/trailz_ui/pkg
dir=$(mktemp -d)
mv $A $B $dir
miniserve -p 9991 $dir

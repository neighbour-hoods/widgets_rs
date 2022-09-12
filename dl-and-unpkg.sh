A=happs-trailz.tar
B=trailz_ui-pkg.tar

dir=$(mktemp -d)
curl 192.168.1.195:9991/$A -o $dir/$A
curl 192.168.1.195:9991/$B -o $dir/$B

tar xvf $dir/$A
tar xvf $dir/$B

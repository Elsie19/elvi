readonly bar="1"
# bar="2"
# comment
baz='boo\tnotab';oof=`ls -l /`
boogle="bar is: '${bar}'"
hash -r
dir="/usr/share"
cd "${dir}"
if [ -f "/etc/resolv.conf" ]; then
    echo "\$PWD is: ${PWD}"
    echo "\$HOME is: ${HOME}"
    echo "Contents of running 'ls /' is ${oof}"
else
    echo "${PATH}"
    dbg PATH
fi

for i in "foo" "bar" "baz"; do
    echo "${i}"
done

echo "My PID is $$ and current process is $0"

for i in ~/Projects/*; do
    echo "${i}"
done

echo "${PATH}"

foo() {
    local foo="bar"
    echo "${0} is running with: ${*}"
}

echo "Count: $#: ${*}"
shift 1
echo "Count: $#: ${*}"
for loopo; do
    echo "${loopo}"
done
foo bing bong
echo "foo is '${foo}'"
if grep -q 'h' /proc/cpuinfo; then
    echo "IT WORKS"
fi

echo "PATH is ${PATH}"
(
    PATH="${PATH}:foo"
    echo "PATH is ${PATH} in a subshell"
)
echo "PATH is back to ${PATH}"
(
    exit 1
)

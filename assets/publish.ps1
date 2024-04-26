git push
if (-not $?)
{
    throw 'Native Failure'
}

cd crate\async-proto-derive
cargo publish
if (-not $?)
{
    throw 'Native Failure'
}

cd ..\async-proto
cargo publish
if (-not $?)
{
    throw 'Native Failure'
}

cd ..\..

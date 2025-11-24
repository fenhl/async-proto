git push
if (-not $?)
{
    throw 'Native Failure'
}

cargo publish --workspace
if (-not $?)
{
    throw 'Native Failure'
}

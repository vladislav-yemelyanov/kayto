cargo set-version --bump patch;
sh cross.sh;
git add .;
git commit -m "upgrade";
git push;
cargo package;
cargo publish;
sleep 1;
curl -fsSL https://raw.githubusercontent.com/vladislav-yemelyanov/kayto/main/install.sh | bash;

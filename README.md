# Make My Code Journal (mmcj)

*Work in Progress*

This project is very much a work in progress and is in no way ready to be used by others. Please keep that in mind if you are veiwing this project. Thank you.

## Goal

I spend alot of time coding and I want a way to summarize all that I have done in one place. Another goal of mine is to document my life better. Thus, the natural idea is to make a code journal that integrates with git to record all the code I have written and combine that with a life journal.

I think it will be cool to see all of this in one place. I'll be able to look back and read my journal and see what I was working on and doing each day.

## Usage

As I said eariler, this is not ready for public usage yet. However, if your curious and have rust installed, you should be able to get it up and running to see what I've done so far.

First, Install Rust If you haven't already. <https://doc.rust-lang.org/stable/book/ch01-01-installation.html#installation>

Then, Clone the repository and run the project.

``` bash
git clone https://github.com/Celnardur/make_my_code_journal.git
cd make_my_code_journal
cargo run
```

If it doesn't run successfully after this, please tell me so I can fix it.

#### Testing

Because this project gets diffs from local repositorys, there needs to be a seperate repository to run tests on. Thus, to run tests you need to clone mmcj_test_repo into the root directory of the project.

``` bash
cd make_my_code_journal
git clone https://github.com/Celnardur/mmcj_test_repo.git
cargo test
```

## Contributing

If you wnat to work with me, just contact me and we'll figure out a way to work together on this.

## License

MIT
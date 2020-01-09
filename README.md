# Make My Code Journal (mmcj)

*Work in Progress*

This project is very much a work in progress and is in no way ready to be used by others. Please keep that in mind if you are veiwing this project. Thank you.

## Goal

I spend alot of time coding and I want a way to summarize all that I have done in one place. Another goal of mine is to document my life better. Thus, the natural idea is to make a code journal that integrates with git to record all the code I have written and combine that with a life journal.

I think it will be cool to see all of this in one place. I'll be able to look back and read my journal and see what I was working on and doing each day.

## Basics

Currently, this takes a list of paths to local repositories and a list of emails from a config file located at  ~/.gitintegratedjournal/sTettings.json. It then uses git2 to filter the commits in those repositories by the emails in the config file. The commits are used to generate diffs, again using git2, and thoses diffs are what are displayed on the screen.

Controlls: j - move up, k - move down, d - expand section, f - collapse section.

Eventually, it will also show more commit information as well as sorting commits by into collapsible days, months and years which will also have summary information. Journal entries will be able to be added directly from the application. I will also add search and sorting capabilities.

All data will be stored locally so you have complete control over your data.

Long team, I might add more personal utilities like a calender, to do lists, notes, etc.

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
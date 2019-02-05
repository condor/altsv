# altsv

An ALTernative LTSV Parser / Dumper gem partially written in Rust.

## Installation

Add this line to your application's Gemfile:

```ruby
gem 'altsv'
```

And then execute:

    $ bundle

Or install it yourself as:

    $ gem install altsv

## Usage

At first, you should require altsv:

    require 'altsv'

In addition, if you manage gems with bundler, you should add the statement below into your Gemfile:

    gem 'altsv'


### parsing LTSV

    # parse string
    string = "label1:value1\tlabel2:value2"
    values = Altsv.parse(string) # => [{:label1 => "value1", :label2 => "value2"}]

    # parse via stream
    # content: as below
    # label1_1:value1_1\tlabel1_2:value1_2
    # label2_1:value2_1\tlabel2_2:value2_2
    stream = File.open("some_file.ltsv", "r")
    values = Altsv.parse(stream)
    # => [{:label1_1 => "value1_2", :label1_2 => "value1_2"},
    #     {:label2_1 => "value2_2", :label2_2 => "value2_2"}]

### loading LTSV file

    # load via path
    values = Altsv.load("some_path.ltsv")

    # load via stream
    stream = File.open("some_file.ltsv", "r")
    values = Altsv.load(stream) # => same as LTSV.parse(stream)

### dumping into LTSV

    value = {label1: "value1", label2: "value2"}
    dumped = Altsv.dump(value) # => "label1:value1\tlabel2:value2"

Dumped objects should respond to :to_hash.

## Development

After checking out the repo, run `bin/setup` to install dependencies. Then, run `rake test` to run the tests. You can also run `bin/console` for an interactive prompt that will allow you to experiment.

To install this gem onto your local machine, run `bundle exec rake install`. To release a new version, update the version number in `version.rb`, and then run `bundle exec rake release`, which will create a git tag for the version, push git commits and tags, and push the `.gem` file to [rubygems.org](https://rubygems.org).

## Contributing

Bug reports and pull requests are welcome on GitHub at https://github.com/condor/altsv.
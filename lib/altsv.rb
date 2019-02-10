# frozen_string_literal: true
require 'helix_runtime'
require 'altsv/native'

require "altsv/version"

class Altsv
  class Error < StandardError; end
  # Your code goes here...

  class << self
    def parse(input)
      case input
      when String
        parse_native input
      when StringIO
        parse_native input.string
      when IO
        input.each_line.map{|line| parse_line_native line}
      end
    end

    def load(path_or_io)
      case path_or_io
      when String
        File.open(path_or_io){|f| parse f}
      when IO
        parse(path_or_io)
      else
        raise ArgumentError, "#{name}.#{__method__} only accepts IO or path."
      end
    end

    def dump(value)
      dump_native value
    end
  end
end

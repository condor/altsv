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
  end
end

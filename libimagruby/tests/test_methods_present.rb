#!/usr/bin/env ruby

require 'imag'
require 'minitest/autorun'

class BasicImagInterfaceTest < Minitest::Test

  def setup
    # nothing
  end

  def test_logger_methods_exist
    [:init, :trace, :dbg, :debug, :info, :warn, :error].each do |m|
      assert Imag::Logger.methods.include? m
    end
  end

end


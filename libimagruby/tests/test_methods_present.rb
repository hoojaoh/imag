#!/usr/bin/env ruby

require 'imag'
require 'minitest/autorun'

class BasicImagInterfaceTest < Minitest::Test

  def setup
    # nothing
  end

  def test_constants_exist
    [ :Logger
    , :StoreId
    , :StoreHandle
    , :FileLockEntryHandle
    , :EntryHeader
    , :EntryContent
    , :VERSION
    ].each do |k|
      assert Imag.constants.include? k
    end
  end

  def test_logger_methods_exist
    [:init, :trace, :dbg, :debug, :info, :warn, :error].each do |m|
      assert Imag::Logger.methods.include? m
    end
  end

end


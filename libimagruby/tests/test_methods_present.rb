#!/usr/bin/env ruby

require 'imag'
require 'minitest/autorun'

class BasicImagInterfaceTest < Minitest::Test

  def setup
    # nothing
  end

  def test_constants_exist
    [ :Logger,
      :StoreId,
      :StoreHandle,
      :FileLockEntryHandle,
      :EntryHeader,
      :EntryContent,
      :VERSION
    ].each do |k|
      assert Imag.constants.include? k
    end
  end

  def test_logger_methods_exist
    [:init, :trace, :dbg, :debug, :info, :warn, :error].each do |m|
      assert Imag::Logger.methods.include? m
    end
  end

  def test_file_lock_entry_handle_methods_exist
    [ :get_location,
      :get_header,
      :set_header,
      :get_content,
      :set_content
    ].each do |m|
      assert Imag::FileLockEntryHandle.methods.include? m
    end
  end

  def test_entry_header_methods_exist
    [ :new,
      :insert,
      :set,
      :get
    ].each do |m|
      assert Imag::EntryHeader.methods.include? m
    end
  end

  def test_store_handle_methods_exist
    [ :new,
      :create,
      :retrieve,
      :get,
      :retrieve_for_module,
      :update,
      :delete,
      :save_to,
      :save_as,
      :move_by_id,
      :path
    ].each do |m|
      assert Imag::StoreHandle.methods.include? m
    end
  end

  def test_storeid_methods_exist
    [ :new,
      :new_baseless,
      :without_base,
      :with_base,
      :into_pathbuf,
      :exists,
      :to_str,
      :local
    ].each do |m|
      assert Imag::StoreId.methods.include? m
    end
  end

end


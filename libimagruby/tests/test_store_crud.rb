#!/usr/bin/env ruby

require 'imag'
require 'minitest/autorun'

class StoreCRUDTest < Minitest::Test

  def setup
    Imag::Logger.init debug: true, verbose: true, color: false
    @store = Imag::StoreHandle.new true, "/tmp/"
  end

  def teardown
    @store = nil
  end

  def test_store_path
    assert_equal "/tmp/store", @store.path
  end

  def test_getting_nonexistent_id
    id = Imag::StoreId::new_baseless "nonexistent"
    assert_raises RImagStoreReadError do
      @store.get id
    end
  end

  def test_create
    id = Imag::StoreId::new_baseless "created"
    fle = @store.create id

    assert_instance_of FileLockEntry, fle
    assert_equal "/tmp/store/created", fle.location
  end

  def test_retrieve_new
    id = Imag::StoreId::new_baseless "retrieved_new"
    fle = @store.retrieve id

    assert_instance_of FileLockEntry, fle
    assert_equal "/tmp/store/retrieved_new", fle.location
  end

  def test_retrieve_old
    id = Imag::StoreId::new_baseless "retrieve_old"
    @store.create id

    fle = @store.retrieve id

    assert_instance_of FileLockEntry, fle
    assert_equal "/tmp/store/retrieved_old", fle.location
  end

  def test_delete_nonexistent
    id = Imag::StoreId::new_baseless "delete_nonexist"

    assert_raises RImagStoreWriteError do
      @store.delete id
    end
  end

  def test_delete_existent
    id = Imag::StoreId::new_baseless "delete_exist"

    @store.create id
    assert_nil @store.delete(id)
  end

end

